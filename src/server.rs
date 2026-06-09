use petgraph::visit::EdgeRef;
use petgraph::graph::NodeIndex;
use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::client::{PublicParameters, Query};
use spiral_rs::params::Params;
use spiral_rs::server::{load_db_from_seek, process_query};
use crate::data_entries::{*};
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
        let packed_db_bytes = GeoServer::build_packed_database(approach, params, &context.graph);
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
        approach: &str,
        params: &Params,
        graph: &EdgeListGraph,
    ) -> Vec<u8> {

        if approach == "node0" {
            
            let mut packed_db = vec![0u8; params.num_items() * params.db_item_size];

            for node_idx in graph.node_indices() {

                let node_entry = Node0Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node3Entry>();
                let end = (node_idx.index() + 1) * std::mem::size_of::<Node3Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "node1" {

            let mut packed_db = vec![0u8; params.num_items() * params.db_item_size];

            for node_idx in graph.node_indices() {

                let node_entry = Node1Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node1Entry>();
                let end = (node_idx.index() + 1) * std::mem::size_of::<Node1Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "node2" {

            let mut packed_db = vec![0u8; params.num_items() * params.db_item_size];

            for node_idx in graph.node_indices() {

                let node_entry = Node2Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node2Entry>();
                let end = (node_idx.index() + 1) * std::mem::size_of::<Node2Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "node3" {

            let mut packed_db = vec![0u8; params.num_items() * params.db_item_size];

            for node_idx in graph.node_indices() {

                let node_entry = Node3Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node3Entry>();
                let end = (node_idx.index() + 1) * std::mem::size_of::<Node3Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else {
            return vec![];
        }

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
