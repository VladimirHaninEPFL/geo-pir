use spiral_rs::aligned_memory::AlignedMemory;
use spiral_rs::client::{PublicParameters, Query};
use spiral_rs::params::Params;
use spiral_rs::server::{load_db_from_seek, process_query};
use crate::db_settings::{Approaches, Architectures, Countries, DBSettings};
use crate::data_entries::{*};
use crate::graph::{EdgeListGraph, GraphContext, GraphResult};
use crate::ipc::{NaiveClientRequest, NaiveServerResponse, SinglePassClientRequest, SinglePassServerResponse, SpiralClientRequest, SpiralServerResponse};
use crate::spiral::{DerivedPirLayout, make_params};

use core::time;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command};
use std::thread::sleep;

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
pub struct GeoServer {
    node_count: usize, // this is for the traffic info
    pub db_settings: DBSettings,

    spiral_settings: Option<SpiralSettings>,
    singlepass_settings: Option<SinglePassSettings>,
    naive_settings: Option<NaiveSettings>,
}

pub struct SinglePassSettings {
    pub stream_child: UnixStream,
    path_socket_server: PathBuf,
}
impl SinglePassSettings {
    pub fn new(db_settings: &DBSettings, graph :&EdgeListGraph, socket_name: &String) -> Self {

        let set_size = (db_settings.logical_db.num_records as f64).sqrt().ceil() as usize;
        let padded_rows = ((db_settings.logical_db.num_records + (set_size - 1)) / set_size) * set_size;

        // here we must create the database for the SinglePass server, its too hard for it to do it
        // we pass the path to the db as command line argument
        let path_db = PathBuf::from(format!("./data/SinglePass-{}-{}.db", db_settings.country.to_string(), db_settings.approach.to_string()));
        if !fs::exists(&path_db).expect("oui") {

            let num_bytes_in_db = db_settings.logical_db.record_size_bytes * padded_rows;
            let packed_db_bytes = GeoServer::build_packed_database(&db_settings, num_bytes_in_db, graph);

            let file = File::create(&path_db).expect("oui");
            let mut writer = BufWriter::new(file);
            writer.write_all(&packed_db_bytes).expect("oui");
        };

        // start the go singlepass server
        let socket_child = PathBuf::from(format!("/tmp/singlepass_private_server_{}-{}-SinglePass-{}.sock", socket_name, db_settings.country.to_string(), db_settings.approach.to_string()));
        let stream_child = SinglePassSettings::spawn_singlepass_server(
            &path_db,
            padded_rows, 
            db_settings.logical_db.record_size_bytes, 
            &socket_child
        );

        let path_socket_server = PathBuf::from(format!("/tmp/{}-SinglePass_{}-{}.sock", db_settings.country.to_string(), socket_name, db_settings.approach.to_string()));

        SinglePassSettings {
            stream_child,
            path_socket_server,
        }
    }

    fn spawn_singlepass_server(db_path: &PathBuf, num_rows: usize, len_rows: usize, socket_path: &PathBuf) -> UnixStream {
        if socket_path.exists() {
            fs::remove_file(socket_path).expect("oui");
        }

        Command::new("./../SinglePass/singlepass-server")         
            .arg(db_path)
            .arg(num_rows.to_string())
            .arg(len_rows.to_string())
            .arg(socket_path)
            .spawn()
            .expect("failed to spawn pir-server");

        // block until you are connected
        loop {
            match UnixStream::connect(&socket_path) {
                Ok(stream) => return stream,
                Err(_) => sleep(time::Duration::from_millis(50)),
            }
        }
    } 
}

pub struct SpiralSettings {
    pub spiral_db: AlignedMemory<64>,
    pub spiral_params: Params,
    pub public_params_bytes: Option<Vec<u8>>,
    pub path_socket_server: PathBuf,
}
impl SpiralSettings {

    pub fn new(db_settings: &DBSettings, graph :&EdgeListGraph) -> Self {

        // spiral setup
        let DerivedPirLayout {
            params,
            records_per_pir_item: _,
        } = make_params(&db_settings.logical_db);

        let path_db = PathBuf::from(format!("./data/Spiral-{}-{}.db", db_settings.country.to_string(), db_settings.approach.to_string()));
        if !fs::exists(&path_db).expect("oui") {

            let num_bytes_in_db = params.num_items() * params.db_item_size;
            let packed_db_bytes = GeoServer::build_packed_database(&db_settings, num_bytes_in_db, &graph);

            let file = File::create(&path_db).expect("oui");
            let mut writer = BufWriter::new(file);
            writer.write_all(&packed_db_bytes).expect("oui");
        }
        
        let file = File::open(&path_db).expect("you must first generate the db !");
        let mut reader = BufReader::new(file);
        let spiral_db = load_db_from_seek(&params, &mut reader);

        let path_socket_server = PathBuf::from(format!("/tmp/{}-Spiral-{}.sock", db_settings.country.to_string(), db_settings.approach.to_string()));

        SpiralSettings {
            spiral_db,
            spiral_params: params,
            public_params_bytes: None,
            path_socket_server,
        }
    }
}

pub struct NaiveSettings {
    pub graph: EdgeListGraph,
    pub path_socket_server: PathBuf,
}

impl GeoServer {
    
    pub fn new(
        country_name: &str,
        architecture_name: &str,
        approach_name: &str,
        socket_name: Option<&String>,
    ) -> GraphResult<Self> {

        let country = country_name
            .parse::<Countries>()
            .expect("unknown country name");
        let context = GraphContext::load(&country)?;

        let db_settings = DBSettings::new(country_name, architecture_name, approach_name, &context.graph);

        match db_settings.architecture {
            Architectures::Spiral => {
                let spiral_settings = SpiralSettings::new(&db_settings, &context.graph);
                Ok(GeoServer {
                    spiral_settings: Some(spiral_settings), 
                    singlepass_settings: None,
                    naive_settings: None,
                    node_count: context.graph.node_count(), 
                    db_settings
                })
            }

            Architectures::SinglePass => {
                let singlepass_settings = SinglePassSettings::new(&db_settings, &context.graph, socket_name.unwrap());
                Ok(GeoServer {
                    spiral_settings: None, 
                    singlepass_settings: Some(singlepass_settings), 
                    naive_settings: None,
                    node_count: context.graph.node_count(), 
                    db_settings
                })
            },
            Architectures::Naive => {
                Ok(GeoServer {
                    spiral_settings: None, 
                    singlepass_settings: None, 
                    node_count: context.graph.node_count(), 
                    naive_settings: Some(NaiveSettings {
                        graph: context.graph, 
                        path_socket_server: PathBuf::from(format!("/tmp/{}-Naive-{}.sock", db_settings.country.to_string(), db_settings.approach.to_string() ))}),
                    db_settings
                })
            }
        }
    }
    
    pub fn run(&mut self) -> io::Result<()> {
        let socket_path: PathBuf ;

        match self.db_settings.architecture {
            Architectures::Spiral => {
                socket_path = self.spiral_settings.as_mut().unwrap().path_socket_server.clone();
            }
            Architectures::SinglePass => {
                socket_path = self.singlepass_settings.as_mut().unwrap().path_socket_server.clone();
            }
            Architectures::Naive => {
                socket_path = self.naive_settings.as_mut().unwrap().path_socket_server.clone();
            }
        }
        
        self.serve_socket(&socket_path)
    }

    fn build_packed_database(
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

        packed_db
    }

    fn serve_socket(&mut self, socket_path: &Path) -> io::Result<()> {
        if socket_path.exists() {
            fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;
        // println!("GeoServer started listening on socket {:?} ...", socket_path);

        // listen to as many clients as needed
        for connection in listener.incoming() {
            // println!("GeoServer accepted new connection to a GeoClient...");

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

        match self.db_settings.architecture {
            Architectures::Spiral => {
                loop {
                    let request = match crate::ipc::receive_message::<SpiralClientRequest>(&mut stream) {
                        Ok(request) => request,
                        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(err) => return Err(err),
                    };

                    let response = self.handle_spiral_request(request);
                    crate::ipc::send_message(&mut stream, &response)?;
                }
            }
            Architectures::SinglePass => {
                loop {
                    let request = match crate::ipc::receive_message::<SinglePassClientRequest>(&mut stream) {
                        Ok(request) => request,
                        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(err) => return Err(err),
                    };

                    let response = self.handle_singlepass_request(request);
                    crate::ipc::send_message(&mut stream, &response)?;
                }
            }
            Architectures::Naive => {
                loop {
                    let request = match crate::ipc::receive_message::<NaiveClientRequest>(&mut stream) {
                        Ok(request) => request,
                        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(err) => return Err(err),
                    };

                    let response = self.handle_naive_request(request);
                    crate::ipc::send_message(&mut stream, &response)?;
                }

            }
        }

        Ok(())
    }
    
    fn handle_naive_request(&mut self, request: NaiveClientRequest) -> NaiveServerResponse {
        match request {
            NaiveClientRequest::GetDBSettings => NaiveServerResponse::DBSettings(self.get_db_settings()),
            NaiveClientRequest::GetCongestion => NaiveServerResponse::Congestion(self.get_congestion()),
            NaiveClientRequest::GetDB => {
                let graph = &self.naive_settings.as_ref().unwrap().graph;
                NaiveServerResponse::DB(bincode::serialize(graph).expect("oui"))
            }
        }
    }

    fn handle_singlepass_request(&mut self, request: SinglePassClientRequest) -> SinglePassServerResponse {
        match request {
            SinglePassClientRequest::GetDBSettings => SinglePassServerResponse::DBSettings(self.get_db_settings()),
            SinglePassClientRequest::GetCongestion => SinglePassServerResponse::Congestion(self.get_congestion()),
            SinglePassClientRequest::GetHints(data) => {
                let result = self.process_singlepass_hints(data);
                SinglePassServerResponse::HintResponse(result)
            }
            SinglePassClientRequest::Query(data) => {
                let result = self.process_singlepass_query(data);
                SinglePassServerResponse::QueryResult(result)
            }
        }
    }

    fn handle_spiral_request(&mut self, request: SpiralClientRequest) -> SpiralServerResponse {
        match request {
            SpiralClientRequest::GetDBSettings => SpiralServerResponse::DBSettings(self.get_db_settings()),
            SpiralClientRequest::GetCongestion => SpiralServerResponse::Congestion(self.get_congestion()),
            SpiralClientRequest::SendPublicParams(bytes) => {
                self.receive_public_params(bytes);
                SpiralServerResponse::Ok
            }
            SpiralClientRequest::Query(data) => {
                let result = self.process_spiral_query(data);
                SpiralServerResponse::QueryResult(result)
            }
        }
    }

    fn process_singlepass_hints(&mut self, hint_request: Vec<u8>) -> Vec<u8> {

        let mut stream_child = &mut self.singlepass_settings.as_mut().unwrap().stream_child;

        // send the type of message first
        let msg_type_data = vec![1_u8];
        crate::ipc::send_data(&mut stream_child, msg_type_data).expect("oui");

        // now send the content
        crate::ipc::send_data(&mut stream_child, hint_request).expect("oui");

        // now read the hint response
        let hint_response = crate::ipc::receive_data(&mut stream_child).expect("oui");

        hint_response
    }

    fn process_singlepass_query(&mut self, query_data: Vec<u8>) -> Vec<u8> {

        let mut stream_child = &mut self.singlepass_settings.as_mut().unwrap().stream_child;

        // send the type of message first
        let msg_type_data = vec![2_u8];
        crate::ipc::send_data(&mut stream_child, msg_type_data).expect("oui");

        // now send the content
        crate::ipc::send_data(&mut stream_child, query_data).expect("oui");

        // now read the repsonse data
        let query_response = crate::ipc::receive_data(&mut stream_child).expect("oui");

        query_response
    }

    fn process_spiral_query(&self, data: Vec<u8>) -> Vec<u8> {
        let spiral_settings = self.spiral_settings.as_ref().unwrap();

        let public_params_bytes = spiral_settings.public_params_bytes.as_ref().expect("public params not set");
        let public_params = PublicParameters::deserialize(&spiral_settings.spiral_params, public_params_bytes);

        let query = Query::deserialize(&spiral_settings.spiral_params, &data);

        let response = process_query(&spiral_settings.spiral_params, &public_params, &query, spiral_settings.spiral_db.as_slice());

        response
    }

    fn get_congestion(&self) -> Vec<u8> {

        let mut congestion: Vec<u16> = vec![];

        // basically, for each node in the graph we write its 4 outgoing edges, even if no edge exists
        for _ in 0..self.node_count {
            for _ in 0..4 {
                congestion.push(1_u16);
            }
        }

        bytemuck::cast_slice(&congestion).to_vec()
    }

    fn get_db_settings(&self) -> Vec<u8> {
        let bytes = self.db_settings.serialize_to_bytes();
        bytes
    }

    fn receive_public_params(&mut self, bytes: Vec<u8>) {
        let spiral_settings = self.spiral_settings.as_mut().unwrap();
        spiral_settings.public_params_bytes = Some(bytes);
    }

}
