use std::env;

use client::Client;
use graph::GraphResult;
use server::Server;

mod client;
mod graph;
mod server;

// struct Params<'a> {
//     country_name: &'a str,
//     approach: &'a str,
//     start_node_osmid: &'a str,
//     end_node_osmid: &'a str,
// }

// #[allow(dead_code)]
// const PARAMS_FRANCE: Params = Params {
//     country_name: "France",
//     approach: "node0",
//     start_node_osmid: "382017",
//     end_node_osmid: "313872541",
// };
// #[allow(dead_code)]
// const PARAMS_SWITZERLAND: Params = Params {
//     country_name: "Switzerland",
//     approach: "node0",
//     start_node_osmid: "312462415",
//     end_node_osmid: "276053614",
// };

fn main() -> GraphResult<()> {

    // parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!(
            "Usage: {} <country_name> <approach> <start_node_osmid> <end_node_osmid>",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = args.get(1).unwrap();
    let approach = args.get(2).unwrap();
    let start_node_osmid = args.get(3).unwrap();
    let end_node_osmid = args.get(4).unwrap();

    // Start the server and run the A* search
    let server = Server::start(country_name, approach)?;
    let mut client = Client::new(server);

    println!(
        "Running A* from {} to {} (client-server architecture) in country {} using approach {}...",
        start_node_osmid, end_node_osmid, country_name, approach
    );

    match client.a_star_search(&start_node_osmid, &end_node_osmid)? {
        Some(result) => {
            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", result.path.len());
            println!("Path: {:?}", result.path);
            println!("Visited nodes: {:?}", result.visited_nodes);
            println!("Number of visited nodes: {}", result.visited_nodes.len());
        }
        None => {
            println!(
                "No path found between {} and {}",
                start_node_osmid, end_node_osmid
            );
        }
    }

    Ok(())
}

#[test]
fn test_all() {
    match main() {
        Ok(()) => (),
        Err(e) => panic!("Test failed with error: {}", e),
    }
}
