mod client;
mod graph;
mod server;

use client::Client;
use graph::{read_graph, GraphResult};
use server::Server;

const EDGELIST_PATH: &str = "data/France-navigation.edgelist";
const NODES_PATH: &str = "data/France-navigation.csv";

const START_NODE_OSMID: &str = "382017";
const END_NODE_OSMID: &str = "313872541";

fn main() -> GraphResult<()> {
    let graph_context = read_graph(EDGELIST_PATH, NODES_PATH)?;

    println!(
        "Loaded graph with {} nodes and {} edges",
        graph_context.graph.node_count(),
        graph_context.graph.edge_count()
    );

    let server = Server::new(graph_context);
    let mut client = Client::new(server);

    println!(
        "Running A* from {} to {} (client-server architecture)...",
        START_NODE_OSMID, END_NODE_OSMID
    );

    match client.a_star_search(START_NODE_OSMID, END_NODE_OSMID)? {
        Some(result) => {
            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", result.path.len());
            println!("Path: {:?}", result.path);
        }
        None => {
            println!(
                "No path found between {} and {}",
                START_NODE_OSMID, END_NODE_OSMID
            );
        }
    }

    Ok(())
}
