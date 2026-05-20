use petgraph::Directed;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::path::Path;

type TravelTime = f64;
type GraphResult<T> = Result<T, Box<dyn Error>>;
type EdgeListGraph = Graph<NodeData, TravelTime, Directed>;

#[derive(Debug, Clone)]
struct NodeData {
    id: String,
    lat: f64,
    lon: f64,
}

const EDGELIST_PATH: &str = "data/flon-navigation.edgelist";
const NODES_PATH: &str = "data/flon-nodes.csv";

fn main() -> GraphResult<()> {
    let graph = read_graph(EDGELIST_PATH, NODES_PATH)?;

    println!(
        "Loaded graph with {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    );
    if let Some(node) = graph.node_weights().next() {
        println!("Example node {} at ({}, {})", node.id, node.lat, node.lon);
    }

    Ok(())
}

fn read_graph(
    edgelist_path: impl AsRef<Path>,
    nodes_path: impl AsRef<Path>,
) -> GraphResult<EdgeListGraph> {

    let mut graph = EdgeListGraph::new();
    let nodes = read_nodes(nodes_path, &mut graph)?;
    read_edges(edgelist_path, &mut graph, &nodes)?;

    Ok(graph)
}

fn read_nodes(
    path: impl AsRef<Path>,
    graph: &mut EdgeListGraph,
) -> GraphResult<HashMap<String, NodeIndex>> {
    let mut nodes = HashMap::<String, NodeIndex>::new();

    let mut reader = csv::Reader::from_path(path)?;
    for record in reader.records() {
        let record = record?;
        let id = record
            .get(0)
            .ok_or_else(|| invalid_data(format!("missing node id in row: {record:?}")))?;
        let lat = parse_csv_float(&record, 1, "lat")?;
        let lon = parse_csv_float(&record, 2, "lon")?;

        let index = graph.add_node(NodeData {
            id: id.to_owned(),
            lat,
            lon,
        });
        nodes.insert(id.to_owned(), index);
    }

    Ok(nodes)
}

fn read_edges(
    path: impl AsRef<Path>,
    graph: &mut EdgeListGraph,
    nodes: &HashMap<String, NodeIndex>,
) -> GraphResult<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.splitn(3, char::is_whitespace);
        let Some(source) = parts.next() else {
            continue;
        };
        let Some(target) = parts.next() else {
            continue;
        };
        let Some(attributes) = parts.next() else {
            return Err(invalid_data(format!("missing edge attributes: {line}")).into());
        };

        let source = find_node(nodes, source)?;
        let target = find_node(nodes, target)?;
        let travel_time = parse_travel_time(attributes)?;

        graph.add_edge(source, target, travel_time);
    }

    Ok(())
}

fn parse_csv_float(record: &csv::StringRecord, index: usize, name: &str) -> io::Result<f64> {
    let value = record
        .get(index)
        .ok_or_else(|| invalid_data(format!("missing {name} in row: {record:?}")))?;

    value
        .parse::<f64>()
        .map_err(|err| invalid_data(format!("invalid {name} value {value:?}: {err}")))
}

fn find_node(nodes: &HashMap<String, NodeIndex>, id: &str) -> io::Result<NodeIndex> {
    nodes
        .get(id)
        .copied()
        .ok_or_else(|| invalid_data(format!("edge references unknown node id: {id}")))
}

fn parse_travel_time(attributes: &str) -> io::Result<TravelTime> {
    let (_, after_key) = attributes
        .split_once("'travelTime':")
        .ok_or_else(|| invalid_data(format!("missing travelTime attribute: {attributes}")))?;

    let value = after_key
        .trim_start()
        .split_once(',')
        .map_or(after_key.trim(), |(value, _)| value.trim());

    value
        .parse::<TravelTime>()
        .map_err(|err| invalid_data(format!("invalid travelTime value {value:?}: {err}")))
}

fn invalid_data(message: String) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}
