use crate::graph::GraphResult;
use eframe::egui;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsNavigation,
};
use petgraph::Directed;
use petgraph::stable_graph::{DefaultIx, StableGraph};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ViewerNode {
    pub osmid: String,
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewerEdge {
    pub source: usize,
    pub target: usize,
}

type OSMGraphView<'a> =
    GraphView<'a, ViewerNode, (), Directed, DefaultIx, DefaultNodeShape, DefaultEdgeShape>;

pub struct ViewerGraph {
    graph: Graph<ViewerNode, (), Directed>,
}

impl ViewerGraph {
    pub fn new(
        nodes: Vec<ViewerNode>,
        edges: Vec<ViewerEdge>,
        visited_nodes: HashSet<String>,
        optimal_path: Vec<String>,
    ) -> Self {
        let mut graph = graph_from_osm(nodes, edges);
        color_graph_nodes(&mut graph, &visited_nodes, &optimal_path);

        Self { graph }
    }
}

pub fn open_graph_viewer(title: String, graph: ViewerGraph) -> GraphResult<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|_cc| Ok(Box::new(GraphViewerApp { graph }))),
    )?;

    Ok(())
}

struct GraphViewerApp {
    graph: ViewerGraph,
}

impl eframe::App for GraphViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} nodes, {} edges",
                self.graph.graph.g().node_count(),
                self.graph.graph.g().edge_count()
            ));
        });
        ui.separator();

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(
                &mut OSMGraphView::new(&mut self.graph.graph)
                    .with_navigations(
                        &SettingsNavigation::default()
                            .with_fit_to_screen_enabled(true)
                            .with_zoom_and_pan_enabled(true),
                    )
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_dragging_enabled(true)
                            .with_node_selection_enabled(true)
                            .with_edge_selection_enabled(true),
                    ),
            );
        });
    }
}

fn graph_from_osm(
    nodes: Vec<ViewerNode>,
    edges: Vec<ViewerEdge>,
) -> Graph<ViewerNode, (), Directed> {
    let locations = projected_locations(&nodes);
    let mut stable_graph = StableGraph::<ViewerNode, (), Directed>::new();
    let node_indices: Vec<_> = nodes
        .into_iter()
        .map(|node| stable_graph.add_node(node))
        .collect();

    for edge in edges {
        stable_graph.add_edge(node_indices[edge.source], node_indices[edge.target], ());
    }

    let mut graph = Graph::from(&stable_graph);
    for (index, location) in locations.into_iter().enumerate() {
        if let Some(node) = graph.g_mut().node_weight_mut(node_indices[index]) {
            node.set_location(location);
            node.set_label(String::new());
        }
    }

    graph
}

fn color_graph_nodes(
    graph: &mut Graph<ViewerNode, (), Directed>,
    visited_nodes: &HashSet<String>,
    optimal_path: &[String],
) {
    let path_nodes: HashSet<&str> = optimal_path.iter().map(String::as_str).collect();
    let node_ids: HashMap<_, _> = graph
        .g()
        .node_indices()
        .filter_map(|index| {
            let node = graph.g().node_weight(index)?;
            Some((index, node.payload().osmid.clone()))
        })
        .collect();

    for (index, osmid) in node_ids {
        let color = if path_nodes.contains(osmid.as_str()) {
            egui::Color32::from_rgb(22, 163, 74)
        } else if visited_nodes.contains(osmid.as_str()) {
            egui::Color32::from_rgb(245, 158, 11)
        } else {
            egui::Color32::from_rgb(220, 38, 38)
        };

        if let Some(node) = graph.g_mut().node_weight_mut(index) {
            node.set_color(color);
        }
    }
}

fn projected_locations(nodes: &[ViewerNode]) -> Vec<egui::Pos2> {
    let Some(first) = nodes.first() else {
        return Vec::new();
    };

    let mut min_lat = first.lat;
    let mut max_lat = first.lat;
    let mut min_lon = first.lon;
    let mut max_lon = first.lon;

    for node in nodes.iter().skip(1) {
        min_lat = min_lat.min(node.lat);
        max_lat = max_lat.max(node.lat);
        min_lon = min_lon.min(node.lon);
        max_lon = max_lon.max(node.lon);
    }

    let lat_range = (max_lat - min_lat).max(f32::EPSILON);
    let lon_range = (max_lon - min_lon).max(f32::EPSILON);
    nodes
        .iter()
        .map(|node| {
            let x = ((node.lon - min_lon) / lon_range) * 1000.0;
            let y = (1.0 - ((node.lat - min_lat) / lat_range)) * 1000.0;
            egui::pos2(x, y)
        })
        .collect()
}
