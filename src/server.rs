use petgraph::visit::EdgeRef;
use petgraph::graph::NodeIndex;
use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::params::Params;
use spiral_rs::server::load_db_from_seek;
use crate::db_params::{LogicalDatabase, Node0Entry, OutgoingEdge, get_logical_db};
use crate::graph::{read_graph, EdgeListGraph, GraphResult, NodeData, TravelTimeEdge};
use crate::spiral::{DerivedPirLayout, make_params};

use std::collections::HashMap;
use std::io::{self, Cursor};

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
/// no osmid are permitted since those are strings
pub struct Server {
    graph: EdgeListGraph,
    spiral_db: AlignedMemory<64>,
}

impl Server {
    pub fn start(
        country_name: &str,
        approach: &str,
        architecture: &str,
        params: &Params,
        logical_db: &LogicalDatabase,
        records_per_pir_item: usize,
    ) -> GraphResult<(Self, HashMap<String, NodeIndex>)> {

        // these files contain the osmid of the nodes and the travel time between them, respectively
        let edgelist_path = format!("./data/{}-navigation.edgelist", country_name);
        let nodes_path = format!("./data/{}-navigation.csv", country_name);

        let context = read_graph(&edgelist_path, &nodes_path)?;

        // spiral setup
        let packed_db_bytes = Server::build_packed_database(params, logical_db, records_per_pir_item, &context.graph);
        let mut packed_db_reader = Cursor::new(packed_db_bytes);
        let spiral_db = load_db_from_seek(&params, &mut packed_db_reader);

        println!(
            "Loaded graph on server with {} nodes and {} edges. Using approach: {} and architecture {}",
            context.graph.node_count(),
            context.graph.edge_count(),
            approach,
            architecture,
        );

        Ok((Server { graph: context.graph, spiral_db }, context.osmid_idx_map))
    }
    
    pub fn build_packed_database(
        params: &Params,
        logical_db: &LogicalDatabase,
        records_per_pir_item: usize,
        graph: &EdgeListGraph,
    ) -> Vec<u8> {

        let mut packed_db = vec![0u8; params.num_items() * params.db_item_size];

        for logical_idx in 0..logical_db.num_records {

            let pir_idx = logical_idx / records_per_pir_item;
            let slot_idx = logical_idx % records_per_pir_item;

            let offset = pir_idx * params.db_item_size + slot_idx * logical_db.record_size_bytes;
            let record_slice = &mut packed_db[offset..offset + logical_db.record_size_bytes];

            let node_data = graph[NodeIndex::new(0)].clone();

            let node_entry: Node0Entry = Node0Entry {
                latitude: node_data.lat,
                longitude: node_data.lon,
                outgoing_edges: [
                    OutgoingEdge { id_target: 0 as u32 + 1, cost: 100, _pad: 0 },
                    OutgoingEdge { id_target: 0 as u32 + 2, cost: 200, _pad: 0 },
                    OutgoingEdge { id_target: 0 as u32 + 3, cost: 300, _pad: 0 },
                    OutgoingEdge { id_target: 0 as u32 + 4, cost: 400, _pad: 0 },
                ],
            };

            record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
        }

        packed_db
    }

    /// Get node information by node_idx
    pub fn get_node_data(&self, node_idx: NodeIndex) -> io::Result<NodeData> {
        Ok(self.graph[node_idx].clone())
    }

    /// Get all outgoing edges from a node (identified by node_idx)
    pub fn get_edges_from(&self, source_node_idx: NodeIndex) -> io::Result<Vec<(NodeIndex, TravelTimeEdge)>> {

        let edges: Vec<(NodeIndex, TravelTimeEdge)> = self.graph
            .edges(source_node_idx)
            .map(|edge| {
                (edge.target(), *edge.weight())
            })
            .collect();

        Ok(edges)
    }
}
