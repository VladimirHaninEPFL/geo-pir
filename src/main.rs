use core::arch;
use std::env;

use client::GeoClient;
use graph::GraphResult;
use server::GeoServer;

mod client;
mod graph;
mod server;
mod spiral;
mod data_entries;

// todo: add traffic information
// todo: improve the packing of nodes in spiral for node0 using their coordinates
// todo: add network usage between client and server 

fn main() -> GraphResult<()> {

    // parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        eprintln!(
            "Usage: {} <country_name> <architecture> <approach> <start_node_osmid> <end_node_osmid>",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = args.get(1).unwrap();
    let architecture = args.get(2).unwrap();
    let approach = args.get(3).unwrap();
    let start_node_osmid = args.get(4).unwrap();
    let end_node_osmid = args.get(5).unwrap();

    let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
    let mut client = GeoClient::new(&mut server, country_name, approach, architecture, &context.graph);

    println!(
        "Running A* from {} to {} in country {} using approach {} and achitecture {} ...",
        start_node_osmid, end_node_osmid, country_name, approach, architecture    
    );

    match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
        Some(result) => {
            println!("A* found a path with cost {:.6}", result.cost);
            println!("Path length: {} nodes", result.path.len());

            println!("Path: {:?}", result.path.iter().map(|graph_idx| {
                context.idx_osmid_map.get(graph_idx).unwrap()
            }).collect::<Vec<_>>());
            println!("Visited nodes: {:?}", result.visited_nodes.iter().map(|graph_idx| {
                context.idx_osmid_map.get(graph_idx).unwrap()
            }).collect::<Vec<_>>());

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
fn test_one_query_node0() -> GraphResult<()> {

    use petgraph::graph::NodeIndex;
    use crate::client::AStarResult;

    let expected_result = AStarResult{
        cost: 0,
        path: vec![NodeIndex::new(101968), NodeIndex::new(216141), NodeIndex::new(439988), NodeIndex::new(439987), NodeIndex::new(58641), NodeIndex::new(403232), NodeIndex::new(35052), NodeIndex::new(35053), NodeIndex::new(35054), NodeIndex::new(45577), NodeIndex::new(45580), NodeIndex::new(35055), NodeIndex::new(403700), NodeIndex::new(403230), NodeIndex::new(403231), NodeIndex::new(301748), NodeIndex::new(301749), NodeIndex::new(203056), NodeIndex::new(203057), NodeIndex::new(347320), NodeIndex::new(347319), NodeIndex::new(216136), NodeIndex::new(203139), NodeIndex::new(203140), NodeIndex::new(216137), NodeIndex::new(87099), NodeIndex::new(13967), NodeIndex::new(13968), NodeIndex::new(35160), NodeIndex::new(347317), NodeIndex::new(185755), NodeIndex::new(100017), NodeIndex::new(100018), NodeIndex::new(105756), NodeIndex::new(105757), NodeIndex::new(12888), NodeIndex::new(100339), NodeIndex::new(108428), NodeIndex::new(108429), NodeIndex::new(103005), NodeIndex::new(100027), NodeIndex::new(68832), NodeIndex::new(68830), NodeIndex::new(68831), NodeIndex::new(305451), NodeIndex::new(305452), NodeIndex::new(305449), NodeIndex::new(101430), NodeIndex::new(101431), NodeIndex::new(101432), NodeIndex::new(35012), NodeIndex::new(35013), NodeIndex::new(35016), NodeIndex::new(35017), NodeIndex::new(232435), NodeIndex::new(232357), NodeIndex::new(401273), NodeIndex::new(35024), NodeIndex::new(253479), NodeIndex::new(417949), NodeIndex::new(424581), NodeIndex::new(250497), NodeIndex::new(250498), NodeIndex::new(250473), NodeIndex::new(250488), NodeIndex::new(250499), NodeIndex::new(250500), NodeIndex::new(401061), NodeIndex::new(314032), NodeIndex::new(425646), NodeIndex::new(425647), NodeIndex::new(250493), NodeIndex::new(250494), NodeIndex::new(406761), NodeIndex::new(406760), NodeIndex::new(223773), NodeIndex::new(223774), NodeIndex::new(234530)],
        visited_nodes: vec![], // don't test this
    };

    let country_name = "Switzerland";
    let architecture = "spiral";
    let approach = "node0";
    let start_node_osmid = "312462415";
    let end_node_osmid = "312462415";

    let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
    let mut client = GeoClient::new(&mut server, approach);

    match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
        Some(result) => {
            assert!(result.cost == expected_result.cost, "the result had cost {} and was expecting {}", result.cost, expected_result.cost);
            assert!(result.path == expected_result.path, "the result had path {:?} and was expecting {:?}", result.path, expected_result.path);
        }
        None => {
            panic!("Should have found path !");
        }
    }

    Ok(())
}

#[test]
fn test_switzerland_node0() -> GraphResult<()> {

    use petgraph::graph::NodeIndex;
    use crate::client::AStarResult;

    let expected_result = AStarResult{
        cost: 524,
        path: vec![NodeIndex::new(101968), NodeIndex::new(216141), NodeIndex::new(439988), NodeIndex::new(439987), NodeIndex::new(58641), NodeIndex::new(403232), NodeIndex::new(35052), NodeIndex::new(35053), NodeIndex::new(35054), NodeIndex::new(45577), NodeIndex::new(45580), NodeIndex::new(35055), NodeIndex::new(403700), NodeIndex::new(403230), NodeIndex::new(403231), NodeIndex::new(301748), NodeIndex::new(301749), NodeIndex::new(203056), NodeIndex::new(203057), NodeIndex::new(347320), NodeIndex::new(347319), NodeIndex::new(216136), NodeIndex::new(203139), NodeIndex::new(203140), NodeIndex::new(216137), NodeIndex::new(87099), NodeIndex::new(13967), NodeIndex::new(13968), NodeIndex::new(35160), NodeIndex::new(347317), NodeIndex::new(185755), NodeIndex::new(100017), NodeIndex::new(100018), NodeIndex::new(105756), NodeIndex::new(105757), NodeIndex::new(12888), NodeIndex::new(100339), NodeIndex::new(108428), NodeIndex::new(108429), NodeIndex::new(103005), NodeIndex::new(100027), NodeIndex::new(68832), NodeIndex::new(68830), NodeIndex::new(68831), NodeIndex::new(305451), NodeIndex::new(305452), NodeIndex::new(305449), NodeIndex::new(101430), NodeIndex::new(101431), NodeIndex::new(101432), NodeIndex::new(35012), NodeIndex::new(35013), NodeIndex::new(35016), NodeIndex::new(35017), NodeIndex::new(232435), NodeIndex::new(232357), NodeIndex::new(401273), NodeIndex::new(35024), NodeIndex::new(253479), NodeIndex::new(417949), NodeIndex::new(424581), NodeIndex::new(250497), NodeIndex::new(250498), NodeIndex::new(250473), NodeIndex::new(250488), NodeIndex::new(250499), NodeIndex::new(250500), NodeIndex::new(401061), NodeIndex::new(314032), NodeIndex::new(425646), NodeIndex::new(425647), NodeIndex::new(250493), NodeIndex::new(250494), NodeIndex::new(406761), NodeIndex::new(406760), NodeIndex::new(223773), NodeIndex::new(223774), NodeIndex::new(234530)],
        visited_nodes: vec![], // don't test this
    };

    let country_name = "Switzerland";
    let architecture = "spiral";
    let approach = "node0";
    let start_node_osmid = "312462415";
    let end_node_osmid = "252684128";

    let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
    let mut client = GeoClient::new(&mut server, approach);

    match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
        Some(result) => {
            assert!(result.cost == expected_result.cost);
            assert!(result.path == expected_result.path);
        }
        None => {
            panic!("Should have found path !");
        }
    }

    Ok(())
}

#[test]
fn test_france_node0() -> GraphResult<()> {

    use petgraph::graph::NodeIndex;
    use crate::client::AStarResult;

    let expected_result = AStarResult{
        cost: 201,
        path: vec![NodeIndex::new(66757), NodeIndex::new(85491), NodeIndex::new(85492), NodeIndex::new(1528289), NodeIndex::new(4950275), NodeIndex::new(1361246), NodeIndex::new(1361245), NodeIndex::new(124627), NodeIndex::new(124628), NodeIndex::new(9035), NodeIndex::new(9039), NodeIndex::new(124629), NodeIndex::new(1250849), NodeIndex::new(122941), NodeIndex::new(4678775), NodeIndex::new(3115773), NodeIndex::new(1250844), NodeIndex::new(1250845), NodeIndex::new(6115), NodeIndex::new(4425460), NodeIndex::new(4553962), NodeIndex::new(3073333)],
        visited_nodes: vec![], // don't test this
    };

    let country_name = "France";
    let architecture = "spiral";
    let approach = "node0";
    let start_node_osmid = "3723996988";
    let end_node_osmid = "2712549945";

    let (mut server, context) = GeoServer::start(country_name, approach, architecture)?;
    let mut client = GeoClient::new(&mut server, approach);

    match client.a_star_search(*context.osmid_idx_map.get(start_node_osmid).unwrap(), *context.osmid_idx_map.get(end_node_osmid).unwrap())? {
        Some(result) => {
            assert!(result.cost == expected_result.cost);
            assert!(result.path == expected_result.path);
        }
        None => {
            panic!("Should have found path !");
        }
    }

    Ok(())
}
