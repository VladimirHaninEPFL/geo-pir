mod client;
mod graph;
mod server;

use client::Client;
use graph::GraphResult;
use server::Server;

struct Params<'a> {
    country_name: &'a str,
    approach: &'a str,
    start_node_osmid: &'a str,
    end_node_osmid: &'a str,
}

#[allow(dead_code)]
const PARAMS_FRANCE: Params = Params {
    country_name: "France",
    approach: "node0",
    start_node_osmid: "382017",
    end_node_osmid: "313872541",
};
#[allow(dead_code)]
const PARAMS_SWITZERLAND: Params = Params {
    country_name: "Switzerland",
    approach: "node0",
    start_node_osmid: "312462415",
    end_node_osmid: "276053614",
};

fn main() -> GraphResult<()> {

    let params = PARAMS_SWITZERLAND; // Change to PARAMS_SWITZERLAND to test with Switzerland data

    let server = Server::start(params.country_name, params.approach)?;
    let mut client = Client::new(server);

    println!(
        "Running A* from {} to {} (client-server architecture)...",
        params.start_node_osmid, params.end_node_osmid
    );

    match client.a_star_search(&params.start_node_osmid, &params.end_node_osmid)? {
        Some(result) => {
            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", result.path.len());
            println!("Path: {:?}", result.path);
            println!("Visited nodes: {:?}", result.visited_nodes);
        }
        None => {
            println!(
                "No path found between {} and {}",
                params.start_node_osmid, params.end_node_osmid
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
