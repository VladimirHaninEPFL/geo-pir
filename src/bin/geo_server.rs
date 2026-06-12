use std::env;
use std::path::PathBuf;

use geo_pir::{graph::GraphResult, server::GeoServer};
use geo_pir::approaches::{parse_approach};


fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 && args.len() != 5 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> [socket_path]",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = &args[1];
    let architecture = &args[2];
    let approach_name = &args[3];
    let socket_path = args.get(4)
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("/tmp/geo_pir.sock"));

    let approach = parse_approach(approach_name);
    let mut server = GeoServer::new(country_name, &approach, architecture)?;

    println!("Starting GeoServer on socket {} ...", socket_path.display());
    server.serve_socket(&socket_path)?;

    Ok(())
}
