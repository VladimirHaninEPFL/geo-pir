use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::client::{PublicParameters, Query};
use spiral_rs::params::Params;
use spiral_rs::server::{load_db_from_seek, process_query};
use crate::db_settings::{Approaches, DBSettings};
use crate::data_entries::{*};
use crate::graph::{EdgeListGraph, GraphResult, read_graph};
use crate::ipc::{ClientRequest, ServerResponse};
use crate::spiral::{DerivedPirLayout, make_params};

use std::fs;
use std::io::{self, Cursor};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
pub struct GeoServer {
    node_count: usize, // this is for the traffic info

    db_settings: DBSettings,

    // this is for spiral
    spiral_db: AlignedMemory<64>,
    pub spiral_params: Params,
    pub public_params_bytes: Option<Vec<u8>>,
}

impl GeoServer {
    
    pub fn new(
        country_name: &str,
        approach_name: &str,
        architecture: &str,
    ) -> GraphResult<Self> {

        // these files contain the osmid of the nodes and the travel time between them, respectively
        let edgelist_path = format!("./data/{}-navigation.edgelist", country_name);
        let nodes_path = format!("./data/{}-navigation.csv", country_name);
        let context = read_graph(&edgelist_path, &nodes_path)?;

        let db_settings = DBSettings::new(country_name, approach_name, architecture, &context.graph);

        // spiral setup
        let DerivedPirLayout {
            params,
            records_per_pir_item: _,
        } = make_params(&db_settings.logical_db);

        let num_bytes_in_db = params.num_items() * params.db_item_size;
        let packed_db_bytes = GeoServer::build_packed_database(&db_settings, num_bytes_in_db, &context.graph);
        let mut packed_db_reader = Cursor::new(packed_db_bytes);
        let spiral_db = load_db_from_seek(&params, &mut packed_db_reader);

        Ok(GeoServer { spiral_db, spiral_params: params, public_params_bytes: None, node_count: context.graph.node_count(), db_settings })
    }
    
    pub fn build_packed_database(
        db_settings: &DBSettings,
        num_bytes_in_db: usize,
        graph: &EdgeListGraph,
    ) -> Vec<u8> {

        println!("Starting creation of database...");

        let mut packed_db = vec![0u8; num_bytes_in_db];

        match db_settings.approach {
            Approaches::Node0 => {
                let entry_size = std::mem::size_of::<Node0Entry>();
                
                for node_idx in graph.node_indices() {

                    let node_entry = Node0Entry::new(graph, node_idx);

                    let offset = node_idx.index() * entry_size;
                    packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
                }
            },
            Approaches::Node1 => {
                let entry_size = std::mem::size_of::<Node1Entry>();
                
                for node_idx in graph.node_indices() {

                    let node_entry = Node1Entry::new(graph, node_idx);

                    let offset = node_idx.index() * entry_size;
                    packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
                }
            },
            Approaches::Node2 => {
                let entry_size = std::mem::size_of::<Node2Entry>();
                
                for node_idx in graph.node_indices() {

                    let node_entry = Node2Entry::new(graph, node_idx);

                    let offset = node_idx.index() * entry_size;
                    packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
                }
            },
            Approaches::Node3 => {
                let entry_size = std::mem::size_of::<Node3Entry>();
                
                for node_idx in graph.node_indices() {

                    let node_entry = Node3Entry::new(graph, node_idx);

                    let offset = node_idx.index() * entry_size;
                    packed_db[offset..offset + entry_size].copy_from_slice(bytemuck::bytes_of(&node_entry));
                }
            },
            Approaches::Block(_) => {

                let block_parameters = db_settings.block_params.as_ref().unwrap();

                // for each block we remember how much it is filled
                let mut next_entry_idx = vec![0usize; block_parameters.num_blocks];

                let block_entry_size = std::mem::size_of::<BlockEntry>();

                for node_idx in graph.node_indices() {

                    let block_node_entry = BlockEntry::new(graph, node_idx);

                    let block_idx = *block_parameters.nodeidx_blockid_map.get(&(node_idx.index() as u32)).unwrap();
                    let entry_index = next_entry_idx[block_idx];

                    assert!(entry_index < block_parameters.nodes_per_block);

                    let start = (block_idx * block_parameters.nodes_per_block  + entry_index) * block_entry_size;
                    let end = start + block_entry_size;
                    packed_db[start..end].copy_from_slice(bytemuck::bytes_of(&block_node_entry));

                    next_entry_idx[block_idx] = entry_index + 1;
                }

            }
        }

        println!("done !");
        packed_db
    }

    pub fn serve_socket(&mut self, socket_path: &Path) -> io::Result<()> {
        if socket_path.exists() {
            fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;

        for connection in listener.incoming() {
            println!("GeoServer accepted new connection to a GeoClient");

            match connection {
                Ok(stream) => {
                    if let Err(err) = self.handle_client(stream) {
                        eprintln!("IPC client handling error: {}", err);
                    }
                }
                Err(err) => {
                    eprintln!("IPC accept error: {}", err);
                }
            }
        }

        Ok(())
    }

    fn handle_client(&mut self, mut stream: UnixStream) -> io::Result<()> {
        loop {
            let request = match crate::ipc::receive_message::<ClientRequest>(&mut stream) {
                Ok(request) => request,
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err),
            };

            let response = self.handle_request(request);
            crate::ipc::send_message(&mut stream, &response)?;
        }

        Ok(())
    }

    fn handle_request(&mut self, request: ClientRequest) -> ServerResponse {
        match request {
            ClientRequest::GetDBSettings => ServerResponse::DBSettings(self.get_db_settings()),
            ClientRequest::SendPublicParams(bytes) => {
                self.receive_public_params(bytes);
                ServerResponse::Ok
            }
            ClientRequest::ProcessQuery(data) => {
                let result = self.process_spiral_query(data);
                ServerResponse::QueryResult(result)
            }
            ClientRequest::GetCongestion => ServerResponse::Congestion(self.get_congestion()),
        }
    }

    pub fn process_spiral_query(&self, data: Vec<u8>) -> Vec<u8> {
        let public_params_bytes = self.public_params_bytes.as_ref().expect("public params not set");
        let public_params = PublicParameters::deserialize(&self.spiral_params, public_params_bytes);

        let query = Query::deserialize(&self.spiral_params, &data);

        let response = process_query(&self.spiral_params, &public_params, &query, self.spiral_db.as_slice());

        response
    }

    pub fn get_congestion(&self) -> Vec<u8> {

        let mut congestion: Vec<u16> = vec![];

        // basically, for each node in the graph we write its 4 outgoing edges, even if no edge exists
        for _ in 0..self.node_count {
            for _ in 0..4 {
                congestion.push(1_u16);
            }
        }

        bytemuck::cast_slice(&congestion).to_vec()
    }

    pub fn get_db_settings(&self) -> Vec<u8> {
        let bytes = self.db_settings.serialize_to_bytes();
        bytes
    }

    pub fn receive_public_params(&mut self, bytes: Vec<u8>) {
        self.public_params_bytes = Some(bytes);
    }
}
