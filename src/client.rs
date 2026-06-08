use petgraph::graph::NodeIndex;
use spiral_rs::{
    client::{Client, PublicParameters, Query},
    params::Params,
    server::{load_db_from_seek, process_query},
    util::get_seeded_rng,
};

use crate::graph::{NodeData, TravelTimeEdge};
use crate::server::Server;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::io::{self, ErrorKind};

type TravelTime = u64; // travel time in seconds used for calculating total path cost

pub struct GeoClient {
    server: Server,

    nodes_cache: HashMap<NodeIndex, NodeData>, // map from node idx to NodeData for caching node information
    edges_cache: HashMap<NodeIndex, Vec<(NodeIndex, TravelTimeEdge)>>, // map from node idx to list of (neighbor_node_idx, travel_time_edge) for caching outgoing edges

    osmid_idx_map: HashMap<String, NodeIndex>, // this is sothat the client can know how to search the 
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

impl GeoClient {
    pub fn new(server: Server, osmid_idx_map: HashMap<String, NodeIndex>) -> Self {
        GeoClient {
            server,
            nodes_cache: HashMap::new(),
            edges_cache: HashMap::new(),
            osmid_idx_map,
        }
    }

    pub fn get_node_index(&self, osmid: &str) -> io::Result<NodeIndex> {
        self.osmid_idx_map
            .get(osmid)
            .copied()
            .ok_or_else(|| invalid_data(format!("unknown node id: {osmid}")))
    }

    /// Get node information, querying the server if not cached
    fn get_node_data(&mut self, node_idx: NodeIndex) -> io::Result<&NodeData> {

        if !self.nodes_cache.contains_key(&node_idx) {
            let node = self.server.get_node_data(node_idx)?;
            self.nodes_cache.insert(node_idx, node);
        }

        Ok(self.nodes_cache.get(&node_idx).unwrap())
    }

    /// Get outgoing edges from a node, querying the server if not cached
    fn get_edges_from(&mut self, node_idx: NodeIndex) -> io::Result<&Vec<(NodeIndex, TravelTimeEdge)>> {

        if !self.edges_cache.contains_key(&node_idx) {
            let edges = self.server.get_edges_from(node_idx)?;
            self.edges_cache.insert(node_idx, edges);
        }

        Ok(self.edges_cache.get(&node_idx).unwrap())
    }

    /// Run A* search from start osmid to goal osmid
    pub fn a_star_search(&mut self, start_osmid: &str, goal_osmid: &str) -> io::Result<Option<AStarResult>> {

        let start_node_idx = self.get_node_index(start_osmid)?;
        let goal_node_idx = self.get_node_index(goal_osmid)?;

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
            for (neighbour_node_idx, travel_time_edge) in neighbors.iter() {

                let proposed_distance = curr_cost + (*travel_time_edge as TravelTime);

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

fn invalid_data(message: String) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}