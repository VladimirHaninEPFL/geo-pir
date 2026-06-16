use petgraph::graph::{NodeIndex};
use spiral_rs::client::{Client, PublicParameters};
use spiral_rs::params::Params;

use crate::db_settings::{Approaches, Architectures, DBSettings};
use crate::{data_entries::*};
use crate::graph::*;
use crate::ipc::ServerHandle;
use crate::spiral::DerivedPirLayout;
use crate::spiral::make_params;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::io::{self};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command};
use std::thread::sleep;
use std::{fs, time};



type TravelTime = u64; // travel time in seconds used for calculating total path cost

#[derive(Debug, Clone)]
pub struct AStarResult {
    pub cost: TravelTime, // total cost of the optimal path found by A*
    pub path: Vec<NodeIndex>, // list of osmids representing the path from start to goal
    pub cached_nodes: Vec<NodeIndex>, // list of osmids of all nodes that were visited during the search (for analysis/visualization)
}

#[derive(Debug, Clone)]
struct AStarState {
    node_idx: NodeIndex,
    f: TravelTime, // heuristic estimate from this node to the goal
    g: TravelTime, // cost from the start node to this node
}
impl Eq for AStarState {}
impl PartialEq for AStarState {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g && self.node_idx == other.node_idx
    }
}
impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_priority = self.f + self.g;
        let other_priority = other.f + other.g;

        // note here we reverse the order to make the BinaryHeap a min-heap based on f+g
        other_priority
            .cmp(&self_priority)
    }
}
impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



pub struct GeoClient<'a> {
    server_handle: ServerHandle,
    server_handle2: Option<ServerHandle>,

    pub db_settings: DBSettings,

    pub nodes_cache: HashMap<NodeIndex, NodeData>, // map from node idx to NodeData for caching node information
    pub edges_cache: HashMap<NodeIndex, Vec<(NodeIndex, TravelTimeEdge)>>, // map from node idx to list of (neighbor_node_idx, travel_time_edge) for caching outgoing edges

    spiral_settings: Option<SpiralSettings<'a>>,
    singlepass_settings: Option<SinglePassSettings>,
}

struct SpiralSettings<'a> {
    spiral_client: Client<'a>,
    records_per_pir_item: usize,
    _params: Box<Params>,
}
impl<'a> SpiralSettings<'a> {

    pub fn new(db_settings: &DBSettings, server_handle: &mut ServerHandle) -> Self {

        let DerivedPirLayout {
            params: params_spiral,
            records_per_pir_item,
        } = make_params(&db_settings.logical_db);

        // Box params so it has a stable heap address that won't change
        // when we move it into the struct.
        let params = Box::new(params_spiral);
        let params_ptr: *const Params = &*params;

        // SAFETY: params is heap-allocated via Box and we never move or drop it
        // before spiral_client. Both live in the returned GeoClient struct,
        // and params_ptr points to the stable Box address, not the stack.
        let mut spiral_client = Client::init(unsafe { &*params_ptr });

        // OFFLINE PHASE OF SPIRAL
        let public_params: PublicParameters = spiral_client.generate_keys();
        SpiralSettings::send_spiral_public_prams(server_handle, &public_params).expect("didn't worked");

        SpiralSettings { spiral_client, records_per_pir_item, _params: params }
    }

    fn send_spiral_public_prams(server_handle: &mut ServerHandle, public_params: &PublicParameters) -> io::Result<()> {
        let bytes: Vec<u8> = public_params.serialize();
        server_handle.send_spiral_public_params(&bytes)
    }
}

struct SinglePassSettings {
    pub stream_child: UnixStream,
}
impl SinglePassSettings {

    pub fn new(db_settings: &DBSettings, server_handle: &mut ServerHandle) -> Self {

        // * start the child spiral client
        let set_size = (db_settings.logical_db.num_records as f64).sqrt().ceil() as usize;
        let socket_child = PathBuf::from(format!("/tmp/singlepass_private_client-{}-SinglePass-{}.sock", db_settings.country.to_string(), db_settings.approach.to_string()));
        let mut stream_child = SinglePassSettings::spawn_singlepass_client(
            set_size,
            &socket_child
        );
        println!("Child singlepass server created !");

        // START THE OFFLINE PHASE OF SINGLEPASS
        // first the client generated a hint request, that we forward to the first server
        let hint_request = crate::ipc::receive_data(&mut stream_child).expect("oui");
        let hint_response = server_handle.send_singlepass_hint_request(&hint_request).expect("uoi");
        crate::ipc::send_data(&mut stream_child, hint_response).expect("oui");

        SinglePassSettings {
            stream_child,
        }
    }

    fn spawn_singlepass_client(set_size: usize, socket_path: &PathBuf) -> UnixStream {
        if socket_path.exists() {
            fs::remove_file(socket_path).expect("oui");
        }

        Command::new("./../SinglePass/singlepass-client")         
            .arg(set_size.to_string())
            .arg(socket_path)
            .spawn()
            .expect("failed to spawn pir-client");

        // block until you are connected
        loop {
            println!("waiting to connect to {:?}", &socket_path);
            match UnixStream::connect(&socket_path) {
                Ok(stream) => return stream,
                Err(_) => sleep(time::Duration::from_millis(50)),
            }
        }
    } 
}


impl<'a> GeoClient<'a> {
    
    pub fn new(
        country_name: &str,
        architecture_name: &str,
        approach_name: &str,
    ) -> GraphResult<Self> {

        let architecture = architecture_name
            .parse::<Architectures>()
            .expect("unknown architecture name");

        match architecture {
            Architectures::Spiral => {

                let socket_path_name = format!("/tmp/{}-{}-{}.sock", country_name, architecture_name, approach_name);
                let server_path = PathBuf::from(&socket_path_name);

                let mut server_handle = ServerHandle::connect(server_path)
                .map_err(|e| format!("Failed to connect to server socket {}: {}", socket_path_name, e))?;

                let db_settings  = GeoClient::get_db_settings_spiral(&mut server_handle)?;
                let spiral_settings = SpiralSettings::new(&db_settings, &mut server_handle);

                Ok(GeoClient {
                    server_handle,
                    server_handle2: None,
                    db_settings,
                    nodes_cache: HashMap::new(),
                    edges_cache: HashMap::new(),

                    spiral_settings: Some(spiral_settings),
                    singlepass_settings: None,
                })
            }
            Architectures::SinglePass => {

                let socket1_path_name = format!("/tmp/{}-{}_left-{}.sock", country_name, architecture_name, approach_name);
                let server1_path = PathBuf::from(&socket1_path_name);
                let mut server1_handle = ServerHandle::connect(server1_path)
                .map_err(|e| format!("Failed to connect to server socket {}: {}", socket1_path_name, e))?;

                let socket2_path_name = format!("/tmp/{}-{}_right-{}.sock", country_name, architecture_name, approach_name);
                let server2_path = PathBuf::from(&socket2_path_name);
                let server2_handle = ServerHandle::connect(server2_path)
                .map_err(|e| format!("Failed to connect to server socket {}: {}", socket2_path_name, e))?;

                let db_settings  = GeoClient::get_db_settings_singlepass(&mut server1_handle)?;

                let singlepass_settings = SinglePassSettings::new(&db_settings, &mut server1_handle);

                Ok(GeoClient {
                    server_handle: server1_handle,
                    server_handle2: Some(server2_handle),
                    db_settings,
                    nodes_cache: HashMap::new(),
                    edges_cache: HashMap::new(),

                    spiral_settings: None,
                    singlepass_settings: Some(singlepass_settings),
                })
            }
        }
    }

    fn get_db_settings_spiral(server_handle: &mut ServerHandle) -> io::Result<DBSettings> {
        let server_bytes = server_handle.get_db_settings_spiral()?;
        let db_settings: DBSettings = DBSettings::deserialize_from_bytes(&server_bytes);

        Ok(db_settings)
    }

    fn get_db_settings_singlepass(server_handle: &mut ServerHandle) -> io::Result<DBSettings> {
        let server_bytes = server_handle.get_db_settings_singlepass()?;
        let db_settings: DBSettings = DBSettings::deserialize_from_bytes(&server_bytes);

        Ok(db_settings)
    }
    
    fn get_congestion(server_handle: &mut ServerHandle) -> io::Result<Vec<u16>> {
        let server_bytes = server_handle.get_congestion()?;
        let congestion_slice: &[u16] = bytemuck::cast_slice(&server_bytes);

        Ok(congestion_slice.to_vec())
    }

    fn send_spiral_query_server(&mut self, data: &Vec<u8>) -> io::Result<Vec<u8>> {
        let server_response = self.server_handle.send_spiral_query(data)?;
        Ok(server_response)
    }

    fn send_singlepass_query_server(&mut self, data: Vec<u8>) -> io::Result<Vec<u8>> {

        let mut stream_child = &self.singlepass_settings.as_ref().unwrap().stream_child;

        // send the index to the spinglepass child
        crate::ipc::send_data(&mut stream_child, data)?;

        // get the two requests byts
        let left_request = crate::ipc::receive_data(&mut stream_child)?;
        let right_request = crate::ipc::receive_data(&mut stream_child)?;

        // send those to the two rust servers IN PARRALEL   
        self.server_handle.send_singlepass_query(&left_request)?;
        self.server_handle2.as_mut().unwrap().send_singlepass_query(&right_request)?;

        let left_response = self.server_handle.get_singlepass_query_repsonse()?;
        let right_response = self.server_handle2.as_mut().unwrap().get_singlepass_query_repsonse()?;

        // send the two responses back for reconstruction
        crate::ipc::send_data(&mut stream_child, left_response)?;
        crate::ipc::send_data(&mut stream_child, right_response)?;
        
        // get the reconstructed row back
        let row = crate::ipc::receive_data(&mut stream_child)?;

        Ok(row)
    }

    fn perform_spiral_request_node(&mut self, node_idx: NodeIndex) -> io::Result<()> {

        let (data, target_idx_clipped) = {
            let spiral_settings = self.spiral_settings.as_mut().unwrap();

            // * spiral query generation for node0
            let target_idx = node_idx.index();
            let target_pir_idx = target_idx / spiral_settings.records_per_pir_item; // this rounds down !
            let target_idx_clipped = target_pir_idx * spiral_settings.records_per_pir_item;

            let query = spiral_settings.spiral_client.generate_query(target_pir_idx);
            let data = query.serialize();

            (data, target_idx_clipped)
        };

        let response = self.send_spiral_query_server(&data)?;

        // * client side response decoding
        let spiral_settings = self.spiral_settings.as_mut().unwrap();
        let result = spiral_settings.spiral_client.decode_response(response.as_slice());

        // you receive multple entries for spiral
        for i in 0..spiral_settings.records_per_pir_item {

            match self.db_settings.approach {
                Approaches::Node0 => {
                    let start = i * std::mem::size_of::<Node0Entry>();
                    let end = (i+1) * std::mem::size_of::<Node0Entry>();
                    let recovered_record = &result[start..end];

                    let node0_entry: &Node0Entry = bytemuck::from_bytes(recovered_record);
                    node0_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
                },
                Approaches::Node1 => {
                    let start = i * std::mem::size_of::<Node1Entry>();
                    let end = (i+1) * std::mem::size_of::<Node1Entry>();
                    let recovered_record = &result[start..end];

                    let node1_entry: &Node1Entry = bytemuck::from_bytes(recovered_record);
                    node1_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
                },
                Approaches::Node2 => {
                    let start = i * std::mem::size_of::<Node2Entry>();
                    let end = (i+1) * std::mem::size_of::<Node2Entry>();
                    let recovered_record = &result[start..end];

                    let node2_entry: &Node2Entry = bytemuck::from_bytes(recovered_record);
                    node2_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
                },
                Approaches::Node3 => {
                    let start = i * std::mem::size_of::<Node3Entry>();
                    let end = (i+1) * std::mem::size_of::<Node3Entry>();
                    let recovered_record = &result[start..end];

                    let node3_entry: &Node3Entry = bytemuck::from_bytes(recovered_record);
                    node3_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
                },
                _ => (),

            }
        }

        Ok(())
    }
    
    fn perform_spiral_request_block(&mut self, node_idx: NodeIndex) -> io::Result<()> {

        let data = {
            let spiral_settings = self.spiral_settings.as_mut().unwrap();
            let block_parameters = self.db_settings.block_params.as_ref().unwrap();
        
            let target_idx = *block_parameters.nodeidx_blockid_map.get(&(node_idx.index() as u32)).unwrap();
            let target_pir_idx = target_idx / spiral_settings.records_per_pir_item; // this rounds down !
            let query = spiral_settings.spiral_client.generate_query(target_pir_idx);
            let data = query.serialize();

            data
        };

        let response = self.send_spiral_query_server(&data)?;

        // * client side response decoding
        let spiral_settings = self.spiral_settings.as_mut().unwrap();
        let block_parameters = self.db_settings.block_params.as_ref().unwrap();

        let result = spiral_settings.spiral_client.decode_response(response.as_slice());

        let num_bytes_in_block = block_parameters.nodes_per_block * std::mem::size_of::<BlockEntry>();

        // you receive multple entries for spiral
        for i in 0..spiral_settings.records_per_pir_item {

            let start = i * num_bytes_in_block;
            let end = start + num_bytes_in_block;
            let block = &result[start..end];

            // extract block content
            let block_entries: &[BlockEntry] = bytemuck::cast_slice(block);
            for entry in block_entries {
                entry.extract_to_graph(self);
            }
        }

        Ok(())
    }

    fn perform_singlepass_request_node(&mut self, node_idx: NodeIndex) -> io::Result<()> {

        let data = (node_idx.index() as u32).to_le_bytes().to_vec();
        let row = self.send_singlepass_query_server(data)?;

        match self.db_settings.approach {
            Approaches::Node0 => {
                let node_entry: &Node0Entry = bytemuck::from_bytes(&row);
                node_entry.extract_to_graph(node_idx,  self);
            },
            Approaches::Node1 => {
                let node_entry: &Node1Entry = bytemuck::from_bytes(&row);
                node_entry.extract_to_graph(node_idx,  self);
            },
            Approaches::Node2 => {
                let node_entry: &Node2Entry = bytemuck::from_bytes(&row);
                node_entry.extract_to_graph(node_idx,  self);
            },
            Approaches::Node3 => {
                let node_entry: &Node3Entry = bytemuck::from_bytes(&row);
                node_entry.extract_to_graph(node_idx,  self);
            },
            _ => {},
        }

        Ok(())
    }
    
    fn perform_singlepass_request_block(&mut self, node_idx: NodeIndex) -> io::Result<()> {

        let block_parameters = self.db_settings.block_params.as_ref().unwrap();
        let target_idx = *block_parameters.nodeidx_blockid_map.get(&(node_idx.index() as u32)).unwrap();

        let data = (target_idx as u32).to_le_bytes().to_vec();
        let row = self.send_singlepass_query_server(data)?;

        // extract block content
        let block_entries: &[BlockEntry] = bytemuck::cast_slice(&row);
        for entry in block_entries {
            entry.extract_to_graph(self);
        }

        Ok(())
    }
    
    /// Get node information, querying the server if not cached
    fn get_node_data(&mut self, node_idx: NodeIndex) -> io::Result<&NodeData> {

        if !self.nodes_cache.contains_key(&node_idx) {

            match self.db_settings.architecture {
                Architectures::Spiral => {
                    match self.db_settings.approach {
                        Approaches::Block(_)    => self.perform_spiral_request_block(node_idx)?,
                        _                       => self.perform_spiral_request_node(node_idx)?
                    }
                },
                Architectures::SinglePass => {
                    match self.db_settings.approach {
                        Approaches::Block(_)    => self.perform_singlepass_request_block(node_idx)?,
                        _                       => self.perform_singlepass_request_node(node_idx)?
                    }
                }
            }
        }

        assert!(self.nodes_cache.contains_key(&node_idx));
        Ok(self.nodes_cache.get(&node_idx).unwrap())
    }

    /// Get outgoing edges from a node
    fn get_edges_from(&mut self, node_idx: NodeIndex) -> io::Result<&Vec<(NodeIndex, TravelTimeEdge)>> {
        assert!(self.edges_cache.contains_key(&node_idx)); // this should never happen ! you must always have known the data of an edge before requesting its out edges

        Ok(self.edges_cache.get(&node_idx).unwrap())
    }

    /// Run A* search from start osmid to goal osmid
    pub fn a_star_search(&mut self, start_node_idx: NodeIndex, goal_node_idx: NodeIndex) -> io::Result<Option<AStarResult>> {

        let congestion = GeoClient::get_congestion(&mut self.server_handle)?;

        let mut best_cost: HashMap<NodeIndex, TravelTime> = HashMap::new(); // this stores the best known cost to reach each node from the start node
        let mut best_source: HashMap<NodeIndex, NodeIndex> = HashMap::new(); // this stores the best known predecessor of each node on the optimal path from the start node (used for path reconstruction)
        let mut open_set = BinaryHeap::new(); // this is the priority queue of nodes to explore, ordered by f = g + h

        // initialize the search with the start node
        best_cost.insert(start_node_idx, 0);
        open_set.push(AStarState {
            node_idx: start_node_idx,
            f: self.heuristic(start_node_idx, goal_node_idx)?,
            g: 0,
        });

        // search loop, until there are no more nodes to explore in the open set
        while let Some(current_state) = open_set.pop() {

            let curr_node_idx  = current_state.node_idx;
            let curr_cost = current_state.g;

            if curr_node_idx == goal_node_idx {
                let path = self.reconstruct_path(&best_source, start_node_idx, goal_node_idx);
                return Ok(Some(AStarResult {
                    cost: curr_cost,
                    path,
                    cached_nodes: self.nodes_cache.keys().cloned().collect(), // Collect visited nodes from the cache keys
                }));
            }

            // this happens when we found a better path to this node
            if curr_cost > *best_cost.get(&curr_node_idx).unwrap_or(&TravelTime::MAX) {
                continue;
            }

            let neighbors = self.get_edges_from(curr_node_idx)?.clone();
            for (i, (neighbour_node_idx, travel_time_edge)) in neighbors.iter().enumerate() {

                let congestion_this_edge = congestion.get(curr_node_idx.index() * 4 + i).unwrap();
                let proposed_distance = curr_cost + (*travel_time_edge as TravelTime + *congestion_this_edge as TravelTime);

                if !best_cost.contains_key(neighbour_node_idx)
                    || proposed_distance < *best_cost.get(neighbour_node_idx).unwrap()
                {
                    best_cost.insert(neighbour_node_idx.clone(), proposed_distance);
                    best_source.insert(neighbour_node_idx.clone(), curr_node_idx.clone());

                    open_set.push(AStarState {
                        node_idx: neighbour_node_idx.clone(),
                        f: self.heuristic(*neighbour_node_idx, goal_node_idx)?,
                        g: proposed_distance,
                    });
                }
            }
        }

        Ok(None)
    }

    fn heuristic(&mut self, from_node_idx: NodeIndex, to_node_idx: NodeIndex) -> io::Result<TravelTime> {
        let from_node = self.get_node_data(from_node_idx)?.clone();
        let to_node = self.get_node_data(to_node_idx)?.clone();

        let distance_meters = haversine_distance_meters(
            from_node.lat,
            from_node.lon,
            to_node.lat,
            to_node.lon,
        );

        Ok(distance_to_seconds(distance_meters))
    }

    fn reconstruct_path(
        &self,
        best_source: &HashMap<NodeIndex, NodeIndex>,
        start: NodeIndex,
        goal: NodeIndex,
    ) -> Vec<NodeIndex> {
        let mut path = Vec::new();
        let mut current = goal;

        while current != start {
            path.push(current.clone());
            if let Some(previous) = best_source.get(&current) {
                current = previous.clone();
            } else {
                return Vec::new();
            }
        }

        path.push(start);
        path.reverse();
        path
    }

}

fn haversine_distance_meters(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    let to_radians = |degrees: f32| degrees.to_radians();
    let phi1 = to_radians(lat1);
    let phi2 = to_radians(lat2);
    let delta_phi = to_radians(lat2 - lat1);
    let delta_lambda = to_radians(lon2 - lon1);

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    const EARTH_RADIUS_METERS: f32 = 6_371_000.0;

    EARTH_RADIUS_METERS * c
}

fn distance_to_seconds(distance_meters: f32) -> TravelTime {
    const CAR_SPEED_MPS: f32 = 130.0_f32 / 3.6_f32; // 130 km/h in meters per second
    (distance_meters / CAR_SPEED_MPS) as TravelTime
}
