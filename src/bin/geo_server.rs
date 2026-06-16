use std::env;
use std::path::PathBuf;

use geo_pir::{graph::GraphResult, server::GeoServer};

// todo: when spiral server answers query, it regenerates the public parameters each time !

fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 && args.len() != 5 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> [left|right]",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = &args[1];
    let architecture_name = &args[2];
    let approach_name = &args[3];
    let socket_name = args.get(4);

    let mut server = GeoServer::new(country_name, architecture_name, approach_name, socket_name)?;

    if socket_name.is_none() {
        let socket_path_name = format!("/tmp/{}-{}-{}.sock", country_name, architecture_name, approach_name);
        println!("Starting GeoServer on socket {} ...", socket_path_name);

        let socket_path = PathBuf::from(socket_path_name);
        server.serve_socket(&socket_path)?;
    } else {
        let socket_path_name = format!("/tmp/{}-{}_{}-{}.sock", country_name, architecture_name, socket_name.unwrap(), approach_name);
        println!("Starting GeoServer on socket {} ...", socket_path_name);

        let socket_path = PathBuf::from(socket_path_name);
        server.serve_socket(&socket_path)?;
    }

    Ok(())
}
