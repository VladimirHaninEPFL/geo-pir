use std::{env, io};

use geo_pir::server::GeoServer;


fn main() -> io::Result<()> {

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

    let mut server =  GeoServer::new(country_name, architecture_name, approach_name, socket_name).expect("oui");
    server.run()
}
