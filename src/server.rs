use petgraph::visit::EdgeRef;

use crate::graph::{read_graph, GraphContext, GraphResult, NodeData, TravelTimeEdge};
use std::io;

/// The server holds the complete graph and serves queries from clients
pub struct Server {
    context: GraphContext,
}

impl Server {
    pub fn start(
        country_name: &str,
        approach: &str,
    ) -> GraphResult<Self> {

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

        Ok(Server { context })
    }

    /// Get node information by osmid
    pub fn get_node(&self, osmid: &str) -> io::Result<NodeData> {
        let index = self.context.get_node_index(osmid)?;
        Ok(self.context.node(index).clone())
    }

    /// Get all outgoing edges from a node (identified by osmid)
    pub fn get_edges_from(&self, osmid: &str) -> io::Result<Vec<(String, TravelTimeEdge)>> {

        let source_idx = self.context.get_node_index(osmid)?;

        let edges: Vec<(String, TravelTimeEdge)> = self.context.graph
            .edges(source_idx)
            .map(|edge| {
                let target_data = self.context.node(edge.target());
                (target_data.osmid.clone(), *edge.weight())
            })
            .collect();
        Ok(edges)
    }
}
