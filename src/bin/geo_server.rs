use std::env;
use std::path::PathBuf;

use geo_pir::{approaches::Approach, graph::GraphResult, server::GeoServer};

fn parse_approach(name: &str) -> Approach<'_> {
    if name.contains("node") {
        Approach {
            name,
            is_node_approach: true,
            block_width: 0.0,
        }
    } else {
        Approach {
            name,
            is_node_approach: false,
            block_width: name
                .trim_start_matches(|c: char| !c.is_ascii_digit())
                .parse()
                .unwrap_or(0.0),
        }
    }
}

fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> <socket_path>",
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

    println!("Starting GeoServer on socket {}", socket_path.display());
    server.serve_socket(&socket_path)?;

    Ok(())
}
