use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::client::{PublicParameters, Query};
use spiral_rs::params::Params;
use spiral_rs::server::{load_db_from_seek, process_query};
use crate::approaches::Approach;
use crate::data_entries::{*};
use crate::graph::{EdgeListGraph, GraphContext, GraphResult, read_graph};
use crate::spiral::{DerivedPirLayout, make_params};

use std::io::Cursor;

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
pub struct GeoServer<'a> {
    node_count: usize, // this is for the traffic info

    // this is for spiral
    spiral_db: AlignedMemory<64>,
    pub params: Params,
    pub public_params: Option<PublicParameters<'a>>,
    pub records_per_pir_item: usize,
}

impl<'a> GeoServer<'a> {
    
    pub fn start(
        country_name: &str,
        approach: &Approach,
        _architecture: &str,
    ) -> GraphResult<(Self, GraphContext)> {

        // these files contain the osmid of the nodes and the travel time between them, respectively
        let edgelist_path = format!("./data/{}-navigation.edgelist", country_name);
        let nodes_path = format!("./data/{}-navigation.csv", country_name);

        let context = read_graph(&edgelist_path, &nodes_path)?;

        let mut block_params: Option<BlockParams> = None;
        if !approach.is_node_approach {
            block_params = Some(get_block_params(&context.graph, approach.block_width));
        }

        let logical_db: LogicalDatabase = get_logical_db(approach, &context.graph, &block_params);

        // spiral setup
        let DerivedPirLayout {
            params,
            records_per_pir_item,
        } = make_params(&logical_db);

        let num_bytes_in_db = params.num_items() * logical_db.record_size_bytes;
        let packed_db_bytes = GeoServer::build_packed_database(approach, num_bytes_in_db, &context.graph, &block_params);
        let mut packed_db_reader = Cursor::new(packed_db_bytes);
        let spiral_db = load_db_from_seek(&params, &mut packed_db_reader);

        Ok((GeoServer { spiral_db, params, public_params: None, records_per_pir_item, node_count: context.graph.node_count() }, context))
    }
    
    pub fn build_packed_database(
        approach: &Approach,
        num_bytes_in_db: usize,
        graph: &EdgeListGraph,
        block_params: &Option<BlockParams>,
    ) -> Vec<u8> {

        let mut packed_db = vec![0u8; num_bytes_in_db];

        if approach.name == "node0" {
            
            let entry_size = std::mem::size_of::<Node0Entry>();
            
            for node_idx in graph.node_indices() {

                let node_entry = Node0Entry::new(graph, node_idx);

                let offset = node_idx.index() * entry_size;
                packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
            }
        }
        else if approach.name == "node1" {

            let entry_size = std::mem::size_of::<Node1Entry>();
            
            for node_idx in graph.node_indices() {

                let node_entry = Node1Entry::new(graph, node_idx);

                let offset = node_idx.index() * entry_size;
                packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
            }
        }
        else if approach.name == "node2" {

            let entry_size = std::mem::size_of::<Node2Entry>();
            
            for node_idx in graph.node_indices() {

                let node_entry = Node1Entry::new(graph, node_idx);

                let offset = node_idx.index() * entry_size;
                packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
            }
        }
        else if approach.name == "node3" {

            let entry_size = std::mem::size_of::<Node3Entry>();
            
            for node_idx in graph.node_indices() {

                let node_entry = Node1Entry::new(graph, node_idx);

                let offset = node_idx.index() * entry_size;
                packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
            }
        }

        else { // you are a block approach

            let block_parameters = block_params.as_ref().unwrap();

            // for each block we remember how much it is filled
            let mut next_entry_idx = vec![0usize; block_parameters.num_blocks];

            let block_entry_size = std::mem::size_of::<BlockEntry>();

            for node_idx in graph.node_indices() {

                let block_node_entry = BlockEntry::new(graph, node_idx);

                let block_idx = *block_parameters.nodeidx_blockid_map.get(&node_idx).unwrap();
                let entry_index = next_entry_idx[block_idx];

                assert!(entry_index < block_parameters.nodes_per_block);

                let start = (block_idx * block_parameters.nodes_per_block  + entry_index) * block_entry_size;
                let end = start + block_entry_size;
                packed_db[start..end].copy_from_slice(bytemuck::bytes_of(&block_node_entry));

                next_entry_idx[block_idx] = entry_index + 1;
            }
        }

        packed_db
    }

    pub fn process_spiral_query(&self, query: &Query) -> Vec<u8> {
        let response = process_query(&self.params, &self.public_params.as_ref().unwrap(), query, self.spiral_db.as_slice());
        response
    }

    pub fn get_congestion(&self) -> Vec<u16> {

        let mut congestion :Vec<u16> = vec![];

        // basically, for each node in the graph we write its 4 outgoing edges, even if no edge exists
        for _ in 0..self.node_count {
            for _ in 0..4 {
                congestion.push(1_u16);
            }
        }

        congestion
    }
}
