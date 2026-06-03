use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::graph::{EdgeListGraph, TravelTime};

#[derive(Debug)]
pub struct AStarResult {
    pub cost: TravelTime,
    pub path: Vec<NodeIndex>,
}

#[derive(Debug, Clone)]
struct State {
    f: TravelTime, // heuristic estimate from this node to the goal
    g: TravelTime, // cost from the start node to this node
    node: NodeIndex,
}

impl Eq for State {}
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g && self.node == other.node
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_priority = self.f + self.g;
        let other_priority = other.f + other.g;

        // here we reverse the order to make the BinaryHeap a min-heap based on the f+g value
        other_priority
            .total_cmp(&self_priority)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn a_star_search(
    graph: &EdgeListGraph,
    start: NodeIndex,
    goal: NodeIndex,
) -> Option<AStarResult> {

    let mut distances: HashMap<NodeIndex, TravelTime> = HashMap::new();
    let mut best_source: HashMap<NodeIndex, NodeIndex> = HashMap::new();
    let mut open_set = BinaryHeap::new();

    distances.insert(start, 0.0);
    open_set.push(State {
        f: heuristic(graph, start, goal),
        g: 0.0,
        node: start,
    });

    while let Some(current_state) = open_set.pop() {
        let curr_node = current_state.node;
        let curr_distance = current_state.g;

        if curr_node == goal {
            let path = reconstruct_path(&best_source, start, goal);
            return Some(AStarResult {
                cost: curr_distance,
                path,
            });
        }

        // This happens if we have already found a better path to curr_node since we added it to the open set, so we can skip processing it again
        if curr_distance > *distances.get(&curr_node).unwrap_or(&f64::INFINITY) {
            continue;
        }

        for edge in graph.edges(curr_node) {
            let neighbor = edge.target();

            let proposed_distance = curr_distance + *edge.weight();

            if !distances.contains_key(&neighbor)
                || proposed_distance < *distances.get(&neighbor).unwrap()
            {
                distances.insert(neighbor, proposed_distance);
                best_source.insert(neighbor, curr_node);

                open_set.push(State {
                    f: heuristic(graph, neighbor, goal),
                    g: proposed_distance,
                    node: neighbor,
                });
            }
        }
    }

    None
}

fn reconstruct_path(
    best_source: &HashMap<NodeIndex, NodeIndex>,
    start: NodeIndex,
    goal: NodeIndex,
) -> Vec<NodeIndex> {
    let mut path = Vec::new();
    let mut current = goal;

    while current != start {
        path.push(current);

        if let Some(&previous) = best_source.get(&current) {
            current = previous;
        } else {
            return Vec::new(); // This should not happen if the path reconstruction is correct, but we handle it just in case
        }
    }

    path.push(start);
    path.reverse();
    path
}

fn heuristic(graph: &EdgeListGraph, node: NodeIndex, goal: NodeIndex) -> TravelTime {
    let origin = &graph[node];
    let destination = &graph[goal];

    let distance_meters = haversine_distance_meters(
        origin.lat,
        origin.lon,
        destination.lat,
        destination.lon,
    );

    distance_to_seconds(distance_meters)
}

fn haversine_distance_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let to_radians = |degrees: f64| degrees.to_radians();
    let phi1 = to_radians(lat1);
    let phi2 = to_radians(lat2);
    let delta_phi = to_radians(lat2 - lat1);
    let delta_lambda = to_radians(lon2 - lon1);

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

    EARTH_RADIUS_METERS * c
}

fn distance_to_seconds(distance_meters: f64) -> TravelTime {
    const CAR_SPEED_MPS: f64 = 130.0_f64 / 3.6_f64; // 130 km/h in meters per second
    distance_meters / CAR_SPEED_MPS
}
