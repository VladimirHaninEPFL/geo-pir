use bincode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::thread::sleep;
use std::time;

#[derive(Serialize, Deserialize, Debug)]
pub enum SpiralClientRequest {
    GetCongestion,
    GetDBSettings,
    Query(Vec<u8>),

    SendPublicParams(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SinglePassClientRequest {
    GetCongestion,
    GetDBSettings,
    Query(Vec<u8>),

    GetHints(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NaiveClientRequest {
    GetCongestion,
    GetDBSettings,
    GetDB,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NaiveServerResponse {
    Congestion(Vec<u8>),
    DBSettings(Vec<u8>),
    Ok,
    Error(String),

    DB(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SinglePassServerResponse {
    Congestion(Vec<u8>),
    DBSettings(Vec<u8>),
    Ok,
    Error(String),

    QueryResult(Vec<u8>),
    HintResponse(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SpiralServerResponse {
    Congestion(Vec<u8>),
    DBSettings(Vec<u8>),
    Ok,
    Error(String),

    QueryResult(Vec<u8>),
}

fn bincode_error_to_io(error: Box<bincode::ErrorKind>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

pub(crate) fn send_data(stream: &mut impl Write, data: Vec<u8>) -> io::Result<()> {
    let len = (data.len() as u32).to_le_bytes();
    stream.write_all(&len)?;
    stream.write_all(&data)?;
    stream.flush()?;
    Ok(())
}

pub(crate) fn send_message<T: Serialize>(stream: &mut impl Write, message: &T) -> io::Result<()> {
    let data = bincode::serialize(message).map_err(bincode_error_to_io)?;
    send_data(stream, data)
}

pub(crate) fn receive_data(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf);

    let mut buffer = vec![0u8; len as usize];
    stream.read_exact(&mut buffer)?;

    Ok(buffer)
}
pub(crate) fn receive_message<T: DeserializeOwned>(stream: &mut impl Read) -> io::Result<T> {
    let data = receive_data(stream)?;
    let message = bincode::deserialize(&data).map_err(bincode_error_to_io)?;

    Ok(message)
}

#[derive(Debug)]
pub struct ServerHandle {
    stream: UnixStream,
}
impl ServerHandle {
    
    pub fn connect<P: AsRef<Path>>(socket_path: P) -> io::Result<Self> {

        // block until you are connected
        loop {
            match UnixStream::connect(&socket_path) {
                Ok(stream) => return Ok(ServerHandle { stream }),
                Err(_) => sleep(time::Duration::from_millis(50)),
            }
        }
    }

    fn spiral_request(&mut self, request: SpiralClientRequest) -> io::Result<SpiralServerResponse> {
        send_message(&mut self.stream, &request)?;
        receive_message(&mut self.stream)
    }

    fn singlepass_request(&mut self, request: SinglePassClientRequest) -> io::Result<SinglePassServerResponse> {
        send_message(&mut self.stream, &request)?;
        receive_message(&mut self.stream)
    }

    fn naive_request(&mut self, request: NaiveClientRequest) -> io::Result<NaiveServerResponse> {
        send_message(&mut self.stream, &request)?;
        receive_message(&mut self.stream)
    }

    pub fn get_db_settings_spiral(&mut self) -> io::Result<Vec<u8>> {
        match self.spiral_request(SpiralClientRequest::GetDBSettings)? {
            SpiralServerResponse::DBSettings(bytes) => Ok(bytes),
            SpiralServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn get_db_settings_singlepass(&mut self) -> io::Result<Vec<u8>> {
        match self.singlepass_request(SinglePassClientRequest::GetDBSettings)? {
            SinglePassServerResponse::DBSettings(bytes) => Ok(bytes),
            SinglePassServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn send_spiral_public_params(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self.spiral_request(SpiralClientRequest::SendPublicParams(bytes.to_vec()))? {
            SpiralServerResponse::Ok => Ok(()),
            SpiralServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn send_singlepass_hint_request(&mut self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        match self.singlepass_request(SinglePassClientRequest::GetHints(bytes.to_vec()))? {
            SinglePassServerResponse::HintResponse(response) => Ok(response),
            SinglePassServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn send_spiral_query(&mut self, query: &[u8]) -> io::Result<Vec<u8>> {
        match self.spiral_request(SpiralClientRequest::Query(query.to_vec()))? {
            SpiralServerResponse::QueryResult(response) => Ok(response),
            SpiralServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn send_naive_query(&mut self) -> io::Result<Vec<u8>> {
        match self.naive_request(NaiveClientRequest::GetDB)? {
            NaiveServerResponse::DB(response) => Ok(response),
            NaiveServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }
    pub fn send_singlepass_query(&mut self, query: &[u8]) -> io::Result<()> {
        send_message(&mut self.stream, &SinglePassClientRequest::Query(query.to_vec()))
    }

    pub fn get_singlepass_query_repsonse(&mut self) -> io::Result<Vec<u8>> {
        match receive_message::<SinglePassServerResponse>(&mut self.stream)? {
            SinglePassServerResponse::QueryResult(response) => Ok(response),
            SinglePassServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }

    pub fn get_congestion(&mut self) -> io::Result<Vec<u8>> {
        match self.spiral_request(SpiralClientRequest::GetCongestion)? {
            SpiralServerResponse::Congestion(bytes) => Ok(bytes),
            SpiralServerResponse::Error(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected response type: {:?}", other),
            )),
        }
    }
}
