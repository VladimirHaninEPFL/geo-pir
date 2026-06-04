use petgraph::visit::EdgeRef;

use crate::graph::{GraphContext, GraphResult, NodeData, TravelTimeEdge, read_graph};
use crate::viewer::{ViewerEdge, ViewerGraph, ViewerNode, open_graph_viewer};
use std::collections::HashSet;
use std::io;

/// The server holds the complete graph and serves queries from clients
pub struct Server {
    context: GraphContext,
}

impl Server {
    pub fn start(country_name: &str, approach: &str) -> GraphResult<Self> {
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

        let edges: Vec<(String, TravelTimeEdge)> = self
            .context
            .graph
            .edges(source_idx)
            .map(|edge| {
                let target_data = self.context.node(edge.target());
                (target_data.osmid.clone(), *edge.weight())
            })
            .collect();
        Ok(edges)
    }

    pub fn open_graph_viewer(
        &self,
        country_name: &str,
        visited_nodes: &HashSet<String>,
        optimal_path: &[String],
    ) -> GraphResult<()> {
        let nodes: Vec<ViewerNode> = self
            .context
            .graph
            .node_indices()
            .map(|index| {
                let node = self.context.node(index);
                ViewerNode {
                    osmid: node.osmid.clone(),
                    lat: node.lat,
                    lon: node.lon,
                }
            })
            .collect();
        let edges: Vec<ViewerEdge> = self
            .context
            .graph
            .edge_references()
            .map(|edge| ViewerEdge {
                source: edge.source().index(),
                target: edge.target().index(),
            })
            .collect();
        let viewer_graph =
            ViewerGraph::new(nodes, edges, visited_nodes.clone(), optimal_path.to_owned());

        open_graph_viewer(format!("{} navigation graph", country_name), viewer_graph)
    }
}
