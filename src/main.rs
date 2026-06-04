mod client;
mod graph;
mod server;

use client::Client;
use graph::{read_graph, GraphResult};
use server::Server;

struct Params<'a> {
    edgelist_path: &'a str,
    nodes_path: &'a str,
    start_node_osmid: &'a str,
    end_node_osmid: &'a str,
}

#[allow(dead_code)]
const PARAMS_FRANCE: Params = Params {
    edgelist_path: "./data/France-navigation.edgelist",
    nodes_path: "./data/France-navigation.csv",
    start_node_osmid: "382017",
    end_node_osmid: "313872541",
};
#[allow(dead_code)]
const PARAMS_SWITZERLAND: Params = Params {
    edgelist_path: "./data/Switzerland-navigation.edgelist",
    nodes_path: "./data/Switzerland-navigation.csv",
    start_node_osmid: "312462415",
    end_node_osmid: "312462415",
};

fn main() -> GraphResult<()> {

    let params = PARAMS_FRANCE; // Change to PARAMS_SWITZERLAND to test with Switzerland data

    let graph_context = read_graph(&params.edgelist_path, &params.nodes_path)?;

    println!(
        "Loaded graph with {} nodes and {} edges",
        graph_context.graph.node_count(),
        graph_context.graph.edge_count()
    );

    let server = Server::new(graph_context);
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