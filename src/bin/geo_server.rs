use std::env;
use std::path::PathBuf;

use geo_pir::{graph::GraphResult, server::GeoServer};

// todo: when spiral server answers query, it regenerates the public parameters each time !

fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> <socket_path>",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = &args[1];
    let architecture_name = &args[2];
    let approach_name = &args[3];
    let socket_path = PathBuf::from(&args[4]);

    let mut server = GeoServer::new(country_name, approach_name, architecture_name)?;

    println!("Starting GeoServer on socket {} ...", socket_path.display());
    server.serve_socket(&socket_path)?;

    Ok(())
}
