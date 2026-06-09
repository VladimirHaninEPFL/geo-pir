use petgraph::visit::EdgeRef;
use petgraph::graph::NodeIndex;
use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::client::{PublicParameters, Query};
use spiral_rs::params::Params;
use spiral_rs::server::{load_db_from_seek, process_query};
use crate::db_params::{LogicalDatabase, Node0Entry, OutgoingEdge, get_logical_db};
use crate::graph::{read_graph, EdgeListGraph, GraphResult, NodeData, TravelTimeEdge};
use crate::spiral::{DerivedPirLayout, make_params};

use std::collections::HashMap;
use std::io::{self, Cursor};

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
/// no osmid are permitted since those are strings
pub struct GeoServer<'a> {
    graph: EdgeListGraph,
    spiral_db: AlignedMemory<64>,
    params: &'a Params,
    public_params: &'a PublicParameters<'a>,
}

impl<'a> GeoServer<'a> {
    
    pub fn start(
        country_name: &str,
        approach: &str,
        architecture: &str,
        params: &'a Params,
        public_params: &'a PublicParameters,
        logical_db: &'a LogicalDatabase,
        records_per_pir_item: usize,
    ) -> GraphResult<(Self, HashMap<String, NodeIndex>, HashMap<NodeIndex, String>)> {

        // these files contain the osmid of the nodes and the travel time between them, respectively
        let edgelist_path = format!("./data/{}-navigation.edgelist", country_name);
        let nodes_path = format!("./data/{}-navigation.csv", country_name);

        let context = read_graph(&edgelist_path, &nodes_path)?;

        // spiral setup
        let packed_db_bytes = GeoServer::build_packed_database(params, logical_db, records_per_pir_item, &context.graph);
        let mut packed_db_reader = Cursor::new(packed_db_bytes);
        let spiral_db = load_db_from_seek(&params, &mut packed_db_reader);

        println!(
            "Loaded graph on server with {} nodes and {} edges. Using approach: {} and architecture {}",
            context.graph.node_count(),
            context.graph.edge_count(),
            approach,
            architecture,
        );

        Ok((GeoServer { graph: context.graph, spiral_db, params, public_params }, context.osmid_idx_map, context.idx_osmid_map))
    }
    
    pub fn build_packed_database(
        params: &Params,
        logical_db: &LogicalDatabase,
        records_per_pir_item: usize,
        graph: &EdgeListGraph,
    ) -> Vec<u8> {

        let mut packed_db = vec![0u8; params.num_items() * params.db_item_size];

        for node_idx in graph.node_indices() {

            let pir_idx = node_idx.index() / records_per_pir_item;
            let slot_idx = node_idx.index() % records_per_pir_item;

            let offset = pir_idx * params.db_item_size + slot_idx * logical_db.record_size_bytes;
            let record_slice = &mut packed_db[offset..offset + logical_db.record_size_bytes];

            let node_data = graph[node_idx].clone();

            let curr_outgoing_edge_entries: Vec<OutgoingEdge> = graph.edges(node_idx).map(|edge| {
                    OutgoingEdge { id_target: edge.target().index() as u32, cost: *edge.weight(), _pad: 0 }
                }).collect();
            if curr_outgoing_edge_entries.len() > 4 {
                println!("Node {:?} has more than 4 outgoing edges ! it has: {}", node_data, curr_outgoing_edge_entries.len());
            }

            let mut outgoing_edge_entries = [OutgoingEdge { id_target: 0, cost: 0, _pad: 0 }; 4];
            for i in 0..4 {
                if i < curr_outgoing_edge_entries.len() {
                    outgoing_edge_entries[i] = curr_outgoing_edge_entries[i]
                }
            };

            let node_entry: Node0Entry = Node0Entry {
                latitude: node_data.lat,
                longitude: node_data.lon,
                outgoing_edges: outgoing_edge_entries
            };
            record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
        }

        packed_db
    }

    pub fn process_spiral_query(&self, query: &Query) -> Vec<u8> {

        let response = process_query(self.params, self.public_params, query, self.spiral_db.as_slice());

        response
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
