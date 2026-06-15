use std::env;

use geo_pir::{client::GeoClient, graph::{GraphContext, GraphResult}};
use petgraph::graph::NodeIndex;


fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 && args.len() != 5 {
        eprintln!(
            "Usage: {} <start_node_osmid> <end_node_osmid> <socket_path> [socket_path2]",
            args[0]
        );
        std::process::exit(1);
    }

    let start_node_osmid = &args[1];
    let end_node_osmid = &args[2];
    let socket_path = &args[3];
    let socket_path2 = args.get(4);

    let mut client = GeoClient::new(socket_path, socket_path2)?;

    // I recalculate this so that I can print the correct values to stdout once I have the result
    let context = GraphContext::load(&client.db_settings.country)?;

    println!(
        "Running A* from {} to {} in country {:?} using approach {:?} and architecture {:?} ...",
        start_node_osmid, end_node_osmid, client.db_settings.country, client.db_settings.approach, client.db_settings.architecture
    );

    let start_node = NodeIndex::new(*context
        .osmid_idx_map
        .get(start_node_osmid)
        .expect("start node not found in graph") as usize);
    let end_node = NodeIndex::new(*context
        .osmid_idx_map
        .get(end_node_osmid)
        .expect("end node not found in graph") as usize);

    match client.a_star_search(start_node, end_node)? {
        Some(result) => {
            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", result.path.len());
            println!(
                "Path: {:?}",
                result
                    .path
                    .iter()
                    .map(|graph_idx| context.idx_osmid_map.get(&(graph_idx.index() as u32)).unwrap())
                    .collect::<Vec<_>>()
            );
            println!(
                "Cached nodes: {:?}",
                result
                    .cached_nodes
                    .iter()
                    .map(|graph_idx| context.idx_osmid_map.get(&(graph_idx.index() as u32)).unwrap())
                    .collect::<Vec<_>>()
            );
            println!("Number of cached nodes: {}", result.cached_nodes.len());
        }
        None => {
            println!("No path found between {} and {}", start_node_osmid, end_node_osmid);
        }
    }

    Ok(())
}