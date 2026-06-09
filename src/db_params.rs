use bytemuck::{Pod, Zeroable};
use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{client::GeoClient, graph::{EdgeListGraph, NodeData}};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Pod, Zeroable)]
pub struct OutgoingEdge {
    pub id_target: u32, // this represents the graph id of the neighbour
    pub cost: u16,
    pub _pad: u16, // explicit padding so that the struct is aligned
}

impl OutgoingEdge {
    pub fn empty() -> Self {
        OutgoingEdge { id_target: 0, cost: 0, _pad: 0 }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node0Entry {
    pub latitude: f32,
    pub longitude: f32,
    pub outgoing_edges: [OutgoingEdge; 4],
}

impl Node0Entry {
        
    pub fn new(graph: &EdgeListGraph, node_idx: NodeIndex) -> Self {

        let outgoing_edges_graph: Vec<OutgoingEdge> = graph.edges(node_idx)
            .map(|edge| {
                OutgoingEdge { id_target: edge.target().index() as u32, cost: *edge.weight(), _pad: 0 }
            })
            .collect();


        let mut outgoing_edges_entries = [OutgoingEdge::empty(); 4];
        for i in 0..4 {
            if i < outgoing_edges_graph.len() {
                outgoing_edges_entries[i] = outgoing_edges_graph[i]
            }
        };

        if outgoing_edges_graph.len() > 4 {
            println!("Node with index {:?} has more than 4 outgoing edges ! it has: {}", node_idx, outgoing_edges_graph.len());
        }

        let node_data = graph[node_idx].clone();

        let node0_entry = Node0Entry {
            latitude: node_data.lat,
            longitude: node_data.lon,
            outgoing_edges: outgoing_edges_entries
        };

        node0_entry
    }

    pub fn empty() -> Self {
        Node0Entry { latitude: 0., longitude: 0., outgoing_edges: [OutgoingEdge::empty(); 4] }
    }

    pub fn extract_to_graph(&self, graph_idx_recovered_record: NodeIndex, geo_client: &mut GeoClient) {

        let node_data = NodeData { lat: self.latitude, lon: self.longitude };
        geo_client.nodes_cache.insert(graph_idx_recovered_record, node_data);

        let outgoing_edges = self.outgoing_edges.iter()
            .filter(|outgoing_edge| **outgoing_edge != OutgoingEdge::empty())
            .map(|outgoing_edge| {
                let neighbour_node_idx = NodeIndex::new(outgoing_edge.id_target as usize);
                let travel_time_edge = outgoing_edge.cost;
                (neighbour_node_idx, travel_time_edge)
            })
            .collect();

        geo_client.edges_cache.insert(graph_idx_recovered_record, outgoing_edges);
    }

}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node1Entry {
    node0_entry: Node0Entry,
    neighbours: [Node0Entry; 4],
}

impl Node1Entry {
        
    pub fn new(graph: &EdgeListGraph, node_idx: NodeIndex) -> Self {

        let outgoing_edges_graph: Vec<Node0Entry> = graph.edges(node_idx)
            .map(|edge| {
                Node0Entry::new(graph, edge.target())
            })
            .collect();

        let mut neighbours = [Node0Entry::empty(); 4];
        for i in 0..4 {
            if i < outgoing_edges_graph.len() {
                neighbours[i] = outgoing_edges_graph[i]
            }
        };

        let node1_entry = Node1Entry {
            node0_entry: Node0Entry::new(graph, node_idx),
            neighbours,
        };

        node1_entry
    }

    pub fn empty() -> Self {
        Node1Entry { node0_entry: Node0Entry::empty(), neighbours: [Node0Entry::empty(); 4] }
    }

    pub fn extract_to_graph(&self, graph_idx_recovered_record: NodeIndex, geo_client: &mut GeoClient) {

        self.node0_entry.extract_to_graph(graph_idx_recovered_record, geo_client);
        
        let neighbour_graph_ids: Vec<NodeIndex> = self.node0_entry.outgoing_edges.iter()
            .filter(|outgoing_edge| {
                **outgoing_edge != OutgoingEdge::empty()
            })
            .map(|outgoing_edge| {
                NodeIndex::new(outgoing_edge.id_target as usize)
            }).collect();

        self.neighbours.iter()
            .zip(neighbour_graph_ids)
            .for_each(|(neighbour_node0_entry, neighbour_graph_idx)| {
                neighbour_node0_entry.extract_to_graph(neighbour_graph_idx, geo_client);
            });
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node2Entry {
    node0_entry: Node0Entry,
    neighbours: [Node1Entry; 4],
}

impl Node2Entry {
        
    pub fn new(graph: &EdgeListGraph, node_idx: NodeIndex) -> Self {

        let outgoing_edges_graph: Vec<Node1Entry> = graph.edges(node_idx)
            .map(|edge| {
                Node1Entry::new(graph, edge.target())
            })
            .collect();

        let mut neighbours = [Node1Entry::empty(); 4];
        for i in 0..4 {
            if i < outgoing_edges_graph.len() {
                neighbours[i] = outgoing_edges_graph[i]
            }
        };

        let node1_entry = Node2Entry {
            node0_entry: Node0Entry::new(graph, node_idx),
            neighbours,
        };

        node1_entry
    }

    pub fn empty() -> Self {
        Node2Entry { node0_entry: Node0Entry::empty(), neighbours: [Node1Entry::empty(); 4] }
    }

    pub fn extract_to_graph(&self, graph_idx_recovered_record: NodeIndex, geo_client: &mut GeoClient) {

        self.node0_entry.extract_to_graph(graph_idx_recovered_record, geo_client);
        
        let neighbour_graph_ids: Vec<NodeIndex> = self.node0_entry.outgoing_edges.iter()
            .filter(|outgoing_edge| {
                **outgoing_edge != OutgoingEdge::empty()
            })
            .map(|outgoing_edge| {
                NodeIndex::new(outgoing_edge.id_target as usize)
            }).collect();

        self.neighbours.iter()
            .zip(neighbour_graph_ids)
            .for_each(|(neighbour_node1_entry, neighbour_graph_idx)| {
                neighbour_node1_entry.extract_to_graph(neighbour_graph_idx, geo_client);
            });
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node3Entry {
    node0_entry: Node0Entry,
    neighbours: [Node2Entry; 4],
}

impl Node3Entry {
        
    pub fn new(graph: &EdgeListGraph, node_idx: NodeIndex) -> Self {

        let outgoing_edges_graph: Vec<Node2Entry> = graph.edges(node_idx)
            .map(|edge| {
                Node2Entry::new(graph, edge.target())
            })
            .collect();

        let mut neighbours = [Node2Entry::empty(); 4];
        for i in 0..4 {
            if i < outgoing_edges_graph.len() {
                neighbours[i] = outgoing_edges_graph[i]
            }
        };

        let node1_entry = Node3Entry {
            node0_entry: Node0Entry::new(graph, node_idx),
            neighbours,
        };

        node1_entry
    }

    pub fn empty() -> Self {
        Node3Entry { node0_entry: Node0Entry::empty(), neighbours: [Node2Entry::empty(); 4] }
    }

    pub fn extract_to_graph(&self, graph_idx_recovered_record: NodeIndex, geo_client: &mut GeoClient) {

        self.node0_entry.extract_to_graph(graph_idx_recovered_record, geo_client);
        
        let neighbour_graph_ids: Vec<NodeIndex> = self.node0_entry.outgoing_edges.iter()
            .filter(|outgoing_edge| {
                **outgoing_edge != OutgoingEdge::empty()
            })
            .map(|outgoing_edge| {
                NodeIndex::new(outgoing_edge.id_target as usize)
            }).collect();

        self.neighbours.iter()
            .zip(neighbour_graph_ids)
            .for_each(|(neighbour_node2_entry, neighbour_graph_idx)| {
                neighbour_node2_entry.extract_to_graph(neighbour_graph_idx, geo_client);
            });
    }
}



pub struct LogicalDatabase {
    pub num_records: usize,
    pub record_size_bytes: usize,
}

pub fn get_logical_db(country_name: &str, approach: &str) -> LogicalDatabase {
    
    // todo: finish this table !
    if country_name == "Switzerland" {
        if approach == "node0" {
            return LogicalDatabase {
                num_records: 467_344,
                record_size_bytes: std::mem::size_of::<Node0Entry>()
            };
        }
        if approach == "node1" {
            return LogicalDatabase {
                num_records: 467_344,
                record_size_bytes: std::mem::size_of::<Node1Entry>()
            };
        }
        if approach == "node2" {
            return LogicalDatabase {
                num_records: 467_344,
                record_size_bytes: std::mem::size_of::<Node2Entry>()
            };
        }
        if approach == "node3" {
            return LogicalDatabase {
                num_records: 467_344,
                record_size_bytes: std::mem::size_of::<Node3Entry>()
            };
        }
        else { // this is wrong
            return LogicalDatabase {
                num_records: 0,
                record_size_bytes: 0,
            };
        }
    }
    else if country_name == "France" {
        if approach == "node0" {
            return LogicalDatabase {
                num_records: 5_196_479,
                record_size_bytes: std::mem::size_of::<Node0Entry>()
            };
        }
        if approach == "node1" {
            return LogicalDatabase {
                num_records: 5_196_479,
                record_size_bytes: std::mem::size_of::<Node1Entry>()
            };
        }
        if approach == "node2" {
            return LogicalDatabase {
                num_records: 5_196_479,
                record_size_bytes: std::mem::size_of::<Node2Entry>()
            };
        }
        if approach == "node3" {
            return LogicalDatabase {
                num_records: 5_196_479,
                record_size_bytes: std::mem::size_of::<Node3Entry>()
            };
        }
        else { // this is wrong
            return LogicalDatabase {
                num_records: 0,
                record_size_bytes: 0,
            };
        }
    }

    else {
        panic!("didn't define parameters for that country");
    }
}

