use petgraph::visit::EdgeRef;

use crate::graph::{GraphContext, NodeData, TravelTime};
use std::io;

/// The server holds the complete graph and serves queries from clients
pub struct Server {
    context: GraphContext,
}

impl Server {
    pub fn new(context: GraphContext) -> Self {
        Server { context }
    }

    /// Get node information by osmid
    pub fn get_node(&self, osmid: &str) -> io::Result<NodeData> {
        let index = self.context.get_node_index(osmid)?;
        Ok(self.context.node(index).clone())
    }

    /// Get all outgoing edges from a node (identified by osmid)
    pub fn get_edges_from(&self, osmid: &str) -> io::Result<Vec<(String, TravelTime)>> {

        let source_idx = self.context.get_node_index(osmid)?;

        let edges: Vec<(String, TravelTime)> = self.context.graph
            .edges(source_idx)
            .map(|edge| {
                let target_data = self.context.node(edge.target());
                (target_data.osmid.clone(), *edge.weight())
            })
            .collect();
        Ok(edges)
    }
}
