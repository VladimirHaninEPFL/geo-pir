use bincode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientRequest {
    SendPublicParams(Vec<u8>),
    ProcessQuery(Vec<u8>),
    GetCongestion,
    GetDBSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse {
    Ok,
    QueryResult(Vec<u8>),
    Congestion(Vec<u8>),
    DBSettings(Vec<u8>),
    Error(String),
}

fn bincode_error_to_io(error: Box<bincode::ErrorKind>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

pub(crate) fn send_message<T: Serialize>(stream: &mut impl Write, message: &T) -> io::Result<()> {
    let data = bincode::serialize(message).map_err(bincode_error_to_io)?;
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&data)?;
    stream.flush()?;
    Ok(())
}

pub(crate) fn receive_message<T: DeserializeOwned>(stream: &mut impl Read) -> io::Result<T> {

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;
    let message = bincode::deserialize(&buffer).map_err(bincode_error_to_io)?;

    Ok(message)
}

#[derive(Debug)]
pub struct ServerHandle {
    stream: UnixStream,
}

impl ServerHandle {
    pub fn connect<P: AsRef<Path>>(socket_path: P) -> io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        Ok(ServerHandle { stream })
    }

    fn request(&mut self, request: ClientRequest) -> io::Result<ServerResponse> {
        send_message(&mut self.stream, &request)?;
        receive_message(&mut self.stream)
    }

    pub fn get_db_settings(&mut self) -> io::Result<Vec<u8>> {
        match self.request(ClientRequest::GetDBSettings)? {
            ServerResponse::DBSettings(bytes) => Ok(bytes),
            ServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn send_public_params(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self.request(ClientRequest::SendPublicParams(bytes.to_vec()))? {
            ServerResponse::Ok => Ok(()),
            ServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn process_query(&mut self, query: &[u8]) -> io::Result<Vec<u8>> {
        match self.request(ClientRequest::ProcessQuery(query.to_vec()))? {
            ServerResponse::QueryResult(response) => Ok(response),
            ServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn get_congestion(&mut self) -> io::Result<Vec<u8>> {
        match self.request(ClientRequest::GetCongestion)? {
            ServerResponse::Congestion(bytes) => Ok(bytes),
            ServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }
}
