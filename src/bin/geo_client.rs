use std::env;

use geo_pir::{client::GeoClient, graph::{GraphContext, GraphResult}};
use petgraph::graph::NodeIndex;


fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> <start_node_osmid> <end_node_osmid>",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = &args[1];
    let architecture_name = &args[2];
    let approach_name = &args[3];
    let start_node_osmid = &args[4];
    let end_node_osmid = &args[5];

    let mut client = GeoClient::new(country_name, architecture_name, approach_name)?;

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

    let maybe_result = client.a_star_search(start_node, end_node)?;

    match maybe_result {
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

            let search_only = result.elapsed_total.checked_sub(result.elapsed_server).unwrap_or_else(|| std::time::Duration::ZERO);
            println!("A* total elapsed time: {:.6} s", result.elapsed_total.as_secs_f64());
            println!("  server queries time: {:.6} s", result.elapsed_server.as_secs_f64());
            println!("  search-only time: {:.6} s", search_only.as_secs_f64());

            println!("Server bytes received: {} bytes", result.server_bytes_received);
        }
        None => {
            println!("!! No path found between {} and {}", start_node_osmid, end_node_osmid);
        }
    }

    Ok(())
}