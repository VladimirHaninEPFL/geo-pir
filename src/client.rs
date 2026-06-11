use petgraph::graph::NodeIndex;
use spiral_rs::client::{Client, PublicParameters};

use crate::approaches::Approach;
use crate::data_entries::*;
use crate::graph::*;
use crate::server::GeoServer;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::io::{self};

type TravelTime = u64; // travel time in seconds used for calculating total path cost

pub struct GeoClient<'a> {
    server: &'a GeoServer<'a>,

    country_name: &'a str,
    approach: &'a Approach<'a>,
    block_params: Option<BlockParams>,

    pub nodes_cache: HashMap<NodeIndex, NodeData>, // map from node idx to NodeData for caching node information
    pub edges_cache: HashMap<NodeIndex, Vec<(NodeIndex, TravelTimeEdge)>>, // map from node idx to list of (neighbor_node_idx, travel_time_edge) for caching outgoing edges

    spiral_client: Client<'a>,
}

#[derive(Debug, Clone)]
pub struct AStarResult {
    pub cost: TravelTime, // total cost of the optimal path found by A*
    pub path: Vec<NodeIndex>, // list of osmids representing the path from start to goal
    pub visited_nodes: Vec<NodeIndex>, // list of osmids of all nodes that were visited during the search (for analysis/visualization)
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

impl<'a> GeoClient<'a> {
    
    pub fn new(
        server: &'a mut GeoServer<'a>, 
        country_name: &'a str,
        approach: &'a Approach,
        _architecture: &str,
        graph: &'a EdgeListGraph,
    ) -> Self {
        
        let mut block_params = None;
        if !approach.is_node_approach {
            block_params = Some(get_block_params(graph, approach.block_width));
        }

        // this is for spiral
        let mut spiral_client = Client::init(&server.params);
        let public_params: PublicParameters = spiral_client.generate_keys();

        server.public_params = Some(public_params); // send public params to the server

        GeoClient {
            server,
            country_name,
            approach,

            block_params,
            nodes_cache: HashMap::new(),
            edges_cache: HashMap::new(),

            spiral_client,
        }
    }

    fn perform_spiral_request_node(&mut self, node_idx: NodeIndex) {

        // * spiral query generation for node0
        let target_idx = node_idx.index();
        let target_pir_idx = target_idx / self.server.records_per_pir_item; // this rounds down !
        let target_idx_clipped = target_pir_idx * self.server.records_per_pir_item;

        let query = self.spiral_client.generate_query(target_pir_idx);
        let response = self.server.process_spiral_query(&query);

        // * client side response decoding
        let result = self.spiral_client.decode_response(response.as_slice());

        // you receive multple entries for spiral
        for i in 0..self.server.records_per_pir_item {

            if self.approach.name == "node0" {

                let start = i * std::mem::size_of::<Node0Entry>();
                let end = (i+1) * std::mem::size_of::<Node0Entry>();
                let recovered_record = &result[start..end];

                let node0_entry: &Node0Entry = bytemuck::from_bytes(recovered_record);
                node0_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
            }
            else if self.approach.name == "node1" {

                let start = i * std::mem::size_of::<Node1Entry>();
                let end = (i+1) * std::mem::size_of::<Node1Entry>();
                let recovered_record = &result[start..end];

                let node1_entry: &Node1Entry = bytemuck::from_bytes(recovered_record);
                node1_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
            }
            else if self.approach.name == "node2" {

                let start = i * std::mem::size_of::<Node2Entry>();
                let end = (i+1) * std::mem::size_of::<Node2Entry>();
                let recovered_record = &result[start..end];

                let node2_entry: &Node2Entry = bytemuck::from_bytes(recovered_record);
                node2_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
            }
            else if self.approach.name == "node3" {

                let start = i * std::mem::size_of::<Node3Entry>();
                let end = (i+1) * std::mem::size_of::<Node3Entry>();
                let recovered_record = &result[start..end];

                let node3_entry: &Node3Entry = bytemuck::from_bytes(recovered_record);
                node3_entry.extract_to_graph(NodeIndex::new(target_idx_clipped + i),  self);
            };
        }

    }
    
    fn perform_spiral_request_block(&mut self, node_idx: NodeIndex) {

        let block_parameters = self.block_params.as_ref().unwrap();

        let target_idx = *block_parameters.nodeidx_blockid_map.get(&node_idx).unwrap();
        let target_pir_idx = target_idx / self.server.records_per_pir_item; // this rounds down !

        let query = self.spiral_client.generate_query(target_pir_idx);

        let response = self.server.process_spiral_query(&query);

        // * client side response decoding
        let result = self.spiral_client.decode_response(response.as_slice());

        let num_bytes_in_block = block_parameters.nodes_per_block * std::mem::size_of::<BlockEntry>();

        // you receive multple entries for spiral
        for i in 0..self.server.records_per_pir_item {

            let start = i * num_bytes_in_block;
            let end = start + num_bytes_in_block;
            let block = &result[start..end];

            // extract block content
            let block_entries: &[BlockEntry] = bytemuck::cast_slice(block);
            for entry in block_entries {
                entry.extract_to_graph(self);
            }
        }
    }

    /// Get node information, querying the server if not cached
    fn get_node_data(&mut self, node_idx: NodeIndex) -> io::Result<&NodeData> {

        if !self.nodes_cache.contains_key(&node_idx) {
            if self.approach.is_node_approach {
                self.perform_spiral_request_node(node_idx);
            } else {
                self.perform_spiral_request_block(node_idx);
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

        let congestion = self.server.get_congestion();

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
                    visited_nodes: self.nodes_cache.keys().cloned().collect(), // Collect visited nodes from the cache keys
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
