use crate::graph::{NodeData, TravelTimeEdge};
use crate::server::Server;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::io;

type TravelTime = u64; // travel time in seconds used for calculating total path cost

pub struct Client {
    server: Server,
    nodes_cache: HashMap<String, NodeData>, // map from osmid to NodeData for caching node information
    edges_cache: HashMap<String, Vec<(String, TravelTimeEdge)>>, // map from osmid to list of (neighbor_osmid, travel_time_edge) for caching outgoing edges
}

#[derive(Debug, Clone)]
pub struct AStarResult {
    pub cost: TravelTime,
    pub path: Vec<String>,
    pub visited_nodes: HashSet<String>,
}

#[derive(Debug, Clone)]
struct AStarState {
    f: TravelTime, // heuristic estimate from this node to the goal
    g: TravelTime, // cost from the start node to this node
    osmid: String,
}

impl Eq for AStarState {}
impl PartialEq for AStarState {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g && self.osmid == other.osmid
    }
}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_priority = self.f + self.g;
        let other_priority = other.f + other.g;

        other_priority.cmp(&self_priority)
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Client {
    pub fn new(server: Server) -> Self {
        Client {
            server,
            nodes_cache: HashMap::new(),
            edges_cache: HashMap::new(),
        }
    }

    /// Get node information, querying the server if not cached
    fn get_node(&mut self, osmid: &str) -> io::Result<&NodeData> {
        if !self.nodes_cache.contains_key(osmid) {
            let node = self.server.get_node(osmid)?;
            self.nodes_cache.insert(osmid.to_string(), node);
        }
        Ok(self.nodes_cache.get(osmid).unwrap())
    }

    /// Get outgoing edges from a node, querying the server if not cached
    fn get_edges_from(&mut self, osmid: &str) -> io::Result<&Vec<(String, TravelTimeEdge)>> {
        if !self.edges_cache.contains_key(osmid) {
            let edges = self.server.get_edges_from(osmid)?;
            self.edges_cache.insert(osmid.to_string(), edges);
        }
        Ok(self.edges_cache.get(osmid).unwrap())
    }

    /// Run A* search from start osmid to goal osmid
    pub fn a_star_search(
        &mut self,
        start_osmid: &str,
        goal_osmid: &str,
    ) -> io::Result<Option<AStarResult>> {
        let mut best_cost: HashMap<String, TravelTime> = HashMap::new();
        let mut best_source: HashMap<String, String> = HashMap::new();
        let mut open_set = BinaryHeap::new();
        let mut visited_nodes = HashSet::new();

        best_cost.insert(start_osmid.to_string(), 0);
        let start_heuristic = self.heuristic(start_osmid, goal_osmid)?;
        open_set.push(AStarState {
            f: start_heuristic,
            g: 0,
            osmid: start_osmid.to_string(),
        });

        while let Some(current_state) = open_set.pop() {
            let curr_osmid = &current_state.osmid;
            let curr_cost = current_state.g;

            if curr_cost > *best_cost.get(curr_osmid).unwrap_or(&TravelTime::MAX) {
                continue;
            }

            visited_nodes.insert(curr_osmid.clone());

            if curr_osmid == goal_osmid {
                let path = self.reconstruct_path(&best_source, start_osmid, goal_osmid);
                return Ok(Some(AStarResult {
                    cost: curr_cost,
                    path,
                    visited_nodes,
                }));
            }

            let neighbors = self.get_edges_from(curr_osmid)?;
            let neighbors = neighbors.clone(); // Clone to release the borrow before calling heuristic
            for (neighbor_osmid, travel_time) in neighbors.iter() {
                let proposed_distance = curr_cost + (*travel_time as TravelTime);

                if !best_cost.contains_key(neighbor_osmid)
                    || proposed_distance < *best_cost.get(neighbor_osmid).unwrap()
                {
                    best_cost.insert(neighbor_osmid.clone(), proposed_distance);
                    best_source.insert(neighbor_osmid.clone(), curr_osmid.clone());

                    let heuristic_estimate = self.heuristic(neighbor_osmid, goal_osmid)?;
                    open_set.push(AStarState {
                        f: heuristic_estimate,
                        g: proposed_distance,
                        osmid: neighbor_osmid.clone(),
                    });
                }
            }
        }

        Ok(None)
    }

    pub fn open_graph_viewer(
        &self,
        country_name: &str,
        visited_nodes: &HashSet<String>,
        optimal_path: &[String],
    ) -> crate::graph::GraphResult<()> {
        self.server
            .open_graph_viewer(country_name, visited_nodes, optimal_path)
    }

    fn heuristic(&mut self, from_osmid: &str, to_osmid: &str) -> io::Result<TravelTime> {
        let from_node = self.get_node(from_osmid)?.clone();
        let to_node = self.get_node(to_osmid)?.clone();

        let distance_meters =
            haversine_distance_meters(from_node.lat, from_node.lon, to_node.lat, to_node.lon);

        Ok(distance_to_seconds(distance_meters))
    }

    fn reconstruct_path(
        &self,
        best_source: &HashMap<String, String>,
        start: &str,
        goal: &str,
    ) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = goal.to_string();

        while current != start {
            path.push(current.clone());
            if let Some(previous) = best_source.get(&current) {
                current = previous.clone();
            } else {
                return Vec::new();
            }
        }

        path.push(start.to_string());
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
