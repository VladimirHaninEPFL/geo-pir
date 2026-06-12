use std::env;
use std::path::PathBuf;

use geo_pir::{approaches::Approach, client::GeoClient, graph::{read_graph, GraphResult}, ipc::ServerHandle};

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
    if args.len() != 6 && args.len() != 7 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> <start_node_osmid> <end_node_osmid> [socket_path]",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = &args[1];
    let architecture = &args[2];
    let approach_name = &args[3];
    let start_node_osmid = &args[4];
    let end_node_osmid = &args[5];
    let socket_path = args
        .get(6)
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("/tmp/geo_pir.sock"));

    let approach = parse_approach(approach_name);
    let context = read_graph(
        format!("./data/{}-navigation.edgelist", country_name),
        format!("./data/{}-navigation.csv", country_name),
    )?;

    let server_handle = ServerHandle::connect(&socket_path)
        .map_err(|e| format!("Failed to connect to server socket {}: {}", socket_path.display(), e))?;
    let mut client = GeoClient::new(server_handle, &approach, architecture, &context.graph)?;

    println!(
        "Running A* from {} to {} in country {} using approach {} and architecture {} ...",
        start_node_osmid, end_node_osmid, country_name, approach_name, architecture
    );

    let start_node = *context
        .osmid_idx_map
        .get(start_node_osmid)
        .expect("start node not found in graph");
    let end_node = *context
        .osmid_idx_map
        .get(end_node_osmid)
        .expect("end node not found in graph");

    match client.a_star_search(start_node, end_node)? {
        Some(result) => {
            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", result.path.len());
            println!(
                "Path: {:?}",
                result
                    .path
                    .iter()
                    .map(|graph_idx| context.idx_osmid_map.get(graph_idx).unwrap())
                    .collect::<Vec<_>>()
            );
            println!(
                "Visited nodes: {:?}",
                result
                    .visited_nodes
                    .iter()
                    .map(|graph_idx| context.idx_osmid_map.get(graph_idx).unwrap())
                    .collect::<Vec<_>>()
            );
            println!("Number of visited nodes: {}", result.visited_nodes.len());
        }
        None => {
            println!("No path found between {} and {}", start_node_osmid, end_node_osmid);
        }
    }

    Ok(())
}

// #[test]
// fn test_one_query_node0() -> GraphResult<()> {

//     use petgraph::graph::NodeIndex;
//     use crate::client::AStarResult;

//     let expected_result = AStarResult{
//         cost: 0,
//         path: vec![NodeIndex::new(101968), NodeIndex::new(216141), NodeIndex::new(439988), NodeIndex::new(439987), NodeIndex::new(58641), NodeIndex::new(403232), NodeIndex::new(35052), NodeIndex::new(35053), NodeIndex::new(35054), NodeIndex::new(45577), NodeIndex::new(45580), NodeIndex::new(35055), NodeIndex::new(403700), NodeIndex::new(403230), NodeIndex::new(403231), NodeIndex::new(301748), NodeIndex::new(301749), NodeIndex::new(203056), NodeIndex::new(203057), NodeIndex::new(347320), NodeIndex::new(347319), NodeIndex::new(216136), NodeIndex::new(203139), NodeIndex::new(203140), NodeIndex::new(216137), NodeIndex::new(87099), NodeIndex::new(13967), NodeIndex::new(13968), NodeIndex::new(35160), NodeIndex::new(347317), NodeIndex::new(185755), NodeIndex::new(100017), NodeIndex::new(100018), NodeIndex::new(105756), NodeIndex::new(105757), NodeIndex::new(12888), NodeIndex::new(100339), NodeIndex::new(108428), NodeIndex::new(108429), NodeIndex::new(103005), NodeIndex::new(100027), NodeIndex::new(68832), NodeIndex::new(68830), NodeIndex::new(68831), NodeIndex::new(305451), NodeIndex::new(305452), NodeIndex::new(305449), NodeIndex::new(101430), NodeIndex::new(101431), NodeIndex::new(101432), NodeIndex::new(35012), NodeIndex::new(35013), NodeIndex::new(35016), NodeIndex::new(35017), NodeIndex::new(232435), NodeIndex::new(232357), NodeIndex::new(401273), NodeIndex::new(35024), NodeIndex::new(253479), NodeIndex::new(417949), NodeIndex::new(424581), NodeIndex::new(250497), NodeIndex::new(250498), NodeIndex::new(250473), NodeIndex::new(250488), NodeIndex::new(250499), NodeIndex::new(250500), NodeIndex::new(401061), NodeIndex::new(314032), NodeIndex::new(425646), NodeIndex::new(425647), NodeIndex::new(250493), NodeIndex::new(250494), NodeIndex::new(406761), NodeIndex::new(406760), NodeIndex::new(223773), NodeIndex::new(223774), NodeIndex::new(234530)],
//         visited_nodes: vec![], // don't test this
//     };

//     let country_name = "Switzerland";
//     let architecture = "spiral";
//     let approach = "node0";
//     let start_node_osmid = "312462415";
//     let end_node_osmid = "312462415";

//     let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
//     let mut client = GeoClient::new(&mut server, approach);

//     match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
//         Some(result) => {
//             assert!(result.cost == expected_result.cost, "the result had cost {} and was expecting {}", result.cost, expected_result.cost);
//             assert!(result.path == expected_result.path, "the result had path {:?} and was expecting {:?}", result.path, expected_result.path);
//         }
//         None => {
//             panic!("Should have found path !");
//         }
//     }

//     Ok(())
// }

// #[test]
// fn test_switzerland_node0() -> GraphResult<()> {

//     use petgraph::graph::NodeIndex;
//     use crate::client::AStarResult;

//     let expected_result = AStarResult{
//         cost: 524,
//         path: vec![NodeIndex::new(101968), NodeIndex::new(216141), NodeIndex::new(439988), NodeIndex::new(439987), NodeIndex::new(58641), NodeIndex::new(403232), NodeIndex::new(35052), NodeIndex::new(35053), NodeIndex::new(35054), NodeIndex::new(45577), NodeIndex::new(45580), NodeIndex::new(35055), NodeIndex::new(403700), NodeIndex::new(403230), NodeIndex::new(403231), NodeIndex::new(301748), NodeIndex::new(301749), NodeIndex::new(203056), NodeIndex::new(203057), NodeIndex::new(347320), NodeIndex::new(347319), NodeIndex::new(216136), NodeIndex::new(203139), NodeIndex::new(203140), NodeIndex::new(216137), NodeIndex::new(87099), NodeIndex::new(13967), NodeIndex::new(13968), NodeIndex::new(35160), NodeIndex::new(347317), NodeIndex::new(185755), NodeIndex::new(100017), NodeIndex::new(100018), NodeIndex::new(105756), NodeIndex::new(105757), NodeIndex::new(12888), NodeIndex::new(100339), NodeIndex::new(108428), NodeIndex::new(108429), NodeIndex::new(103005), NodeIndex::new(100027), NodeIndex::new(68832), NodeIndex::new(68830), NodeIndex::new(68831), NodeIndex::new(305451), NodeIndex::new(305452), NodeIndex::new(305449), NodeIndex::new(101430), NodeIndex::new(101431), NodeIndex::new(101432), NodeIndex::new(35012), NodeIndex::new(35013), NodeIndex::new(35016), NodeIndex::new(35017), NodeIndex::new(232435), NodeIndex::new(232357), NodeIndex::new(401273), NodeIndex::new(35024), NodeIndex::new(253479), NodeIndex::new(417949), NodeIndex::new(424581), NodeIndex::new(250497), NodeIndex::new(250498), NodeIndex::new(250473), NodeIndex::new(250488), NodeIndex::new(250499), NodeIndex::new(250500), NodeIndex::new(401061), NodeIndex::new(314032), NodeIndex::new(425646), NodeIndex::new(425647), NodeIndex::new(250493), NodeIndex::new(250494), NodeIndex::new(406761), NodeIndex::new(406760), NodeIndex::new(223773), NodeIndex::new(223774), NodeIndex::new(234530)],
//         visited_nodes: vec![], // don't test this
//     };

//     let country_name = "Switzerland";
//     let architecture = "spiral";
//     let approach = "node0";
//     let start_node_osmid = "312462415";
//     let end_node_osmid = "252684128";

//     let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
//     let mut client = GeoClient::new(&mut server, approach);

//     match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
//         Some(result) => {
//             assert!(result.cost == expected_result.cost);
//             assert!(result.path == expected_result.path);
//         }
//         None => {
//             panic!("Should have found path !");
//         }
//     }

//     Ok(())
// }

// #[test]
// fn test_france_node0() -> GraphResult<()> {

//     use petgraph::graph::NodeIndex;
//     use crate::client::AStarResult;

//     let expected_result = AStarResult{
//         cost: 201,
//         path: vec![NodeIndex::new(66757), NodeIndex::new(85491), NodeIndex::new(85492), NodeIndex::new(1528289), NodeIndex::new(4950275), NodeIndex::new(1361246), NodeIndex::new(1361245), NodeIndex::new(124627), NodeIndex::new(124628), NodeIndex::new(9035), NodeIndex::new(9039), NodeIndex::new(124629), NodeIndex::new(1250849), NodeIndex::new(122941), NodeIndex::new(4678775), NodeIndex::new(3115773), NodeIndex::new(1250844), NodeIndex::new(1250845), NodeIndex::new(6115), NodeIndex::new(4425460), NodeIndex::new(4553962), NodeIndex::new(3073333)],
//         visited_nodes: vec![], // don't test this
//     };

//     let country_name = "France";
//     let architecture = "spiral";
//     let approach = "node0";
//     let start_node_osmid = "3723996988";
//     let end_node_osmid = "2712549945";

//     let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
//     let mut client = GeoClient::new(&mut server, approach);

//     match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
//         Some(result) => {
//             assert!(result.cost == expected_result.cost);
//             assert!(result.path == expected_result.path);
//         }
//         None => {
//             panic!("Should have found path !");
//         }
//     }

//     Ok(())
// }
