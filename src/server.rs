use petgraph::visit::EdgeRef;
use petgraph::graph::NodeIndex;
use crate::graph::{read_graph, EdgeListGraph, GraphResult, NodeData, TravelTimeEdge};
use std::collections::HashMap;
use std::io;

/// The server holds the complete graph and serves queries from clients
/// note that the server here has to receive the graph node idx (or the db index of the appraoch)
/// no osmid are permitted since those are strings
pub struct Server {
    graph: EdgeListGraph,
}

impl Server {
    pub fn start(
        country_name: &str,
        approach: &str,
    ) -> GraphResult<(Self, HashMap<String, NodeIndex>)> {

        // these files contain the osmid of the nodes and the travel time between them, respectively
        let edgelist_path = format!("./data/{}-navigation.edgelist", country_name);
        let nodes_path = format!("./data/{}-navigation.csv", country_name);

        let context = read_graph(&edgelist_path, &nodes_path)?;

        println!(
            "Loaded graph on server with {} nodes and {} edges. Using approach: {}",
            context.graph.node_count(),
            context.graph.edge_count(),
            approach
        );

        Ok((Server { graph: context.graph }, context.osmid_idx_map))
    }

    /// Get node information by node_idx
    pub fn get_node_data(&self, node_idx: NodeIndex) -> io::Result<NodeData> {
        Ok(self.graph[node_idx].clone())
    }

    /// Get all outgoing edges from a node (identified by node_idx)
    pub fn get_edges_from(&self, source_node_idx: NodeIndex) -> io::Result<Vec<(NodeIndex, TravelTimeEdge)>> {

        let edges: Vec<(NodeIndex, TravelTimeEdge)> = self.graph
            .edges(source_node_idx)
            .map(|edge| {
                (edge.target(), *edge.weight())
            })
            .collect();

        Ok(edges)
    }
}
