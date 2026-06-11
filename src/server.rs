use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::client::{PublicParameters, Query};
use spiral_rs::params::Params;
use spiral_rs::server::{load_db_from_seek, process_query};
use crate::data_entries::{*};
use crate::graph::{EdgeListGraph, GraphContext, GraphResult, read_graph};
use crate::spiral::{DerivedPirLayout, make_params};
use petgraph::visit::EdgeRef;

use std::io::Cursor;

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
pub struct GeoServer<'a> {
    spiral_db: AlignedMemory<64>,
    pub params: Params,
    pub public_params: Option<PublicParameters<'a>>,
    pub records_per_pir_item: usize,
    graph: EdgeListGraph,
}

impl<'a> GeoServer<'a> {
    
    pub fn start(
        country_name: &str,
        approach: &str,
        _architecture: &str,
    ) -> GraphResult<(Self, GraphContext)> {

        // these files contain the osmid of the nodes and the travel time between them, respectively
        let edgelist_path = format!("./data/{}-navigation.edgelist", country_name);
        let nodes_path = format!("./data/{}-navigation.csv", country_name);

        let context = read_graph(&edgelist_path, &nodes_path)?;

        let logical_db: LogicalDatabase = get_logical_db(country_name, approach);

        // spiral setup
        let DerivedPirLayout {
            params,
            records_per_pir_item,
        } = make_params(&logical_db);

        let num_bytes_in_db = params.num_items() * params.db_item_size;
        let packed_db_bytes = GeoServer::build_packed_database(approach, num_bytes_in_db, &context.graph, &logical_db);
        let mut packed_db_reader = Cursor::new(packed_db_bytes);
        let spiral_db = load_db_from_seek(&params, &mut packed_db_reader);

        Ok((GeoServer { spiral_db, params, public_params: None, records_per_pir_item, graph: context.graph.clone() }, context))
    }
    
    pub fn build_packed_database(
        approach: &str,
        num_bytes_in_db: usize,
        graph: &EdgeListGraph,
        logical_db: &LogicalDatabase,
    ) -> Vec<u8> {

        if approach == "node0" {
            
            let mut packed_db = vec![0u8; num_bytes_in_db];

            for node_idx in graph.node_indices() {

                let node_entry = Node0Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node0Entry>();
                let end = start + std::mem::size_of::<Node0Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "node1" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            for node_idx in graph.node_indices() {

                let node_entry = Node1Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node0Entry>();
                let end = start + std::mem::size_of::<Node0Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "node2" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            for node_idx in graph.node_indices() {

                let node_entry = Node2Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node0Entry>();
                let end = start + std::mem::size_of::<Node0Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "node3" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            for node_idx in graph.node_indices() {

                let node_entry = Node3Entry::new(graph, node_idx);

                let start = node_idx.index() * std::mem::size_of::<Node3Entry>();
                let end = start + std::mem::size_of::<Node0Entry>();
                let record_slice = &mut packed_db[start..end];
                record_slice.copy_from_slice(bytemuck::bytes_of(&node_entry));
            }

            return packed_db;
        }
        else if approach == "block01" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            let node_blockid_map = get_node_blockid_map(graph, 0.1);

            let block_entry_size = std::mem::size_of::<BlockEntry>();
            let block_capacity = logical_db.record_size_bytes / block_entry_size;
            let mut next_entry_idx = vec![0usize; logical_db.num_records];

            for node_idx in graph.node_indices() {

                let block_entry = BlockEntry::new(graph, node_idx);

                let block_id = *node_blockid_map.get(&node_idx).unwrap() as usize;
                assert!(block_id < logical_db.num_records, "block_id exceeds logical database block count");

                let entry_index = next_entry_idx[block_id];
                assert!(entry_index < block_capacity, "block {} has more entries than configured capacity", block_id);

                let start = block_id * logical_db.record_size_bytes + entry_index * block_entry_size;
                let end = start + block_entry_size;
                packed_db[start..end].copy_from_slice(bytemuck::bytes_of(&block_entry));

                next_entry_idx[block_id] = entry_index + 1;
            }

            return packed_db;
        }
        else if approach == "block025" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            let node_blockid_map = get_node_blockid_map(graph, 0.25);

            let block_entry_size = std::mem::size_of::<BlockEntry>();
            let block_capacity = logical_db.record_size_bytes / block_entry_size;
            let mut next_entry_idx = vec![0usize; logical_db.num_records];

            for node_idx in graph.node_indices() {

                let block_entry = BlockEntry::new(graph, node_idx);

                let block_id = *node_blockid_map.get(&node_idx).unwrap() as usize;
                assert!(block_id < logical_db.num_records, "block_id exceeds logical database block count");

                let entry_index = next_entry_idx[block_id];
                assert!(entry_index < block_capacity, "block {} has more entries than configured capacity", block_id);

                let start = block_id * logical_db.record_size_bytes + entry_index * block_entry_size;
                let end = start + block_entry_size;
                packed_db[start..end].copy_from_slice(bytemuck::bytes_of(&block_entry));

                next_entry_idx[block_id] = entry_index + 1;
            }

            return packed_db;
        }
        else if approach == "block05" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            let node_blockid_map = get_node_blockid_map(graph, 0.5);

            let block_entry_size = std::mem::size_of::<BlockEntry>();
            let block_capacity = logical_db.record_size_bytes / block_entry_size;
            let mut next_entry_idx = vec![0usize; logical_db.num_records];

            for node_idx in graph.node_indices() {

                let block_entry = BlockEntry::new(graph, node_idx);

                let block_id = *node_blockid_map.get(&node_idx).unwrap() as usize;
                assert!(block_id < logical_db.num_records, "block_id exceeds logical database block count");

                let entry_index = next_entry_idx[block_id];
                assert!(entry_index < block_capacity, "block {} has more entries than configured capacity", block_id);

                let start = block_id * logical_db.record_size_bytes + entry_index * block_entry_size;
                let end = start + block_entry_size;
                packed_db[start..end].copy_from_slice(bytemuck::bytes_of(&block_entry));

                next_entry_idx[block_id] = entry_index + 1;
            }

            return packed_db;
        }
        else if approach == "block1" {

            let mut packed_db = vec![0u8; num_bytes_in_db];

            let node_blockid_map = get_node_blockid_map(graph, 1.);

            let block_entry_size = std::mem::size_of::<BlockEntry>();
            let block_capacity = logical_db.record_size_bytes / block_entry_size;
            let mut next_entry_idx = vec![0usize; logical_db.num_records];

            for node_idx in graph.node_indices() {

                let block_entry = BlockEntry::new(graph, node_idx);

                let block_id = *node_blockid_map.get(&node_idx).unwrap() as usize;
                assert!(block_id < logical_db.num_records, "block_id exceeds logical database block count");

                let entry_index = next_entry_idx[block_id];
                assert!(entry_index < block_capacity, "block {} has more entries than configured capacity", block_id);

                let start = block_id * logical_db.record_size_bytes + entry_index * block_entry_size;
                let end = start + block_entry_size;
                packed_db[start..end].copy_from_slice(bytemuck::bytes_of(&block_entry));

                next_entry_idx[block_id] = entry_index + 1;
            }

            return packed_db;
        }
        else {
            return vec![];
        }

    }

    pub fn process_spiral_query(&self, query: &Query) -> Vec<u8> {
        let response = process_query(&self.params, &self.public_params.as_ref().unwrap(), query, self.spiral_db.as_slice());
        response
    }

    pub fn get_congestion(&self) -> Vec<u16> {

        let mut congestion :Vec<u16> = vec![];

        // basically, for each node in the graph we write its 4 outgoing edges, even if no edge exists
        for _ in 0..self.graph.node_count() {
            for _ in 0..4 {
                congestion.push(1_u16);
            }
        }

        congestion
    }
}
