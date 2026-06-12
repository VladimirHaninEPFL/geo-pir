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

        let mut outgoing_edges = [OutgoingEdge::empty(); 4];
        for (edge_index, edge) in graph.edges(node_idx).enumerate().take(4) {
            outgoing_edges[edge_index] = OutgoingEdge {
                id_target: edge.target().index() as u32,
                cost: *edge.weight(),
                _pad: 0,
            };
        }

        let node_data = graph[node_idx].clone();

        let node0_entry = Node0Entry {
            latitude: node_data.lat,
            longitude: node_data.lon,
            outgoing_edges: outgoing_edges
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

        let mut neighbours = [Node0Entry::empty(); 4];
        for (edge_index, edge) in graph.edges(node_idx).enumerate().take(4) {
            neighbours[edge_index] = Node0Entry::new(graph, edge.target());
        }

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

        let mut neighbours = [Node1Entry::empty(); 4];
        for (edge_index, edge) in graph.edges(node_idx).enumerate().take(4) {
            neighbours[edge_index] = Node1Entry::new(graph, edge.target());
        }

        let node2_entry = Node2Entry {
            node0_entry: Node0Entry::new(graph, node_idx),
            neighbours,
        };

        node2_entry
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

        let mut neighbours = [Node2Entry::empty(); 4];
        for (edge_index, edge) in graph.edges(node_idx).enumerate().take(4) {
            neighbours[edge_index] = Node2Entry::new(graph, edge.target());
        }

        let node3_entry = Node3Entry {
            node0_entry: Node0Entry::new(graph, node_idx),
            neighbours,
        };

        node3_entry
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


// block approaches
pub type BlockId = usize;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BlockEntry { // very similar to the node0 entries, we just add the current node id
    pub node_id: u32,

    pub latitude: f32,
    pub longitude: f32,
    pub outgoing_edges: [OutgoingEdge; 4],
}

impl BlockEntry {

    // pub fn empty() -> Self {
    //     BlockEntry { node_id: 0, latitude: 0., longitude: 0., outgoing_edges: [OutgoingEdge::empty(); 4] }
    // }

    pub fn new(graph: &EdgeListGraph, node_idx: NodeIndex) -> Self {

        let node_data = &graph[node_idx];

        let mut outgoing_edges = [OutgoingEdge::empty(); 4];
        for (edge_index, edge) in graph.edges(node_idx).enumerate().take(4) {
            outgoing_edges[edge_index] = OutgoingEdge {
                id_target: edge.target().index() as u32,
                cost: *edge.weight(),
                _pad: 0,
            };
        }

        let block_entry = BlockEntry {
            node_id: node_idx.index() as u32,
            latitude: node_data.lat,
            longitude: node_data.lon,
            outgoing_edges,
        };

        block_entry
    }

    pub fn extract_to_graph(&self, geo_client: &mut GeoClient) {

        let node_data = NodeData { lat: self.latitude, lon: self.longitude };
        geo_client.nodes_cache.insert(NodeIndex::new(self.node_id as usize), node_data);

        let outgoing_edges = self.outgoing_edges.iter()
            .filter(|outgoing_edge| **outgoing_edge != OutgoingEdge::empty())
            .map(|outgoing_edge| {
                let neighbour_node_idx = NodeIndex::new(outgoing_edge.id_target as usize);
                let travel_time_edge = outgoing_edge.cost;
                (neighbour_node_idx, travel_time_edge)
            })
            .collect();

        geo_client.edges_cache.insert(NodeIndex::new(self.node_id as usize), outgoing_edges);
    }

}
