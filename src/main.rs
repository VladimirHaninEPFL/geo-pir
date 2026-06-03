mod astar;
mod graph;

use astar::a_star_search;
use graph::{read_graph, GraphResult};

const EDGELIST_PATH: &str = "data/France-navigation.edgelist";
const NODES_PATH: &str = "data/France-navigation.csv";
const START_NODE_OSMID: &str = "382017";
const END_NODE_OSMID: &str = "15378841";

fn main() -> GraphResult<()> {
    let context = read_graph(EDGELIST_PATH, NODES_PATH)?;

    println!(
        "Loaded graph with {} nodes and {} edges",
        context.graph.node_count(),
        context.graph.edge_count()
    );

    let start = context.get_node_index(START_NODE_OSMID)?;
    let end = context.get_node_index(END_NODE_OSMID)?;

    println!(
        "Running A* from {} to {}...",
        context.node(start).osmid,
        context.node(end).osmid
    );

    match a_star_search(&context.graph, start, end) {
        Some(result) => {
            let path_ids: Vec<_> = result.path
                .iter()
                .map(|node| context.node(*node).osmid.clone())
                .collect();

            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", path_ids.len());
            println!("Path: {:?}", path_ids);
        }
        None => {
            println!("No path found between {START_NODE_OSMID} and {END_NODE_OSMID}");
        }
    }

    Ok(())
}
