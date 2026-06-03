use petgraph::graph::{Graph, NodeIndex};
use petgraph::Directed;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::path::Path;

pub type TravelTime = f64;
pub type GraphResult<T> = Result<T, Box<dyn Error>>;
pub type EdgeListGraph = Graph<NodeData, TravelTime, Directed>;

#[derive(Debug, Clone)]
pub struct NodeData {
    pub osmid: String,
    pub lat: f64,
    pub lon: f64,
}

pub struct GraphContext {
    pub graph: EdgeListGraph,
    pub osmid_idx_map: HashMap<String, NodeIndex>, // here we map the OSM node IDs to their corresponding NodeIndex in the graph for easy lookup when reading edges
}

impl GraphContext {
    pub fn get_node_index(&self, osmid: &str) -> io::Result<NodeIndex> {
        self.osmid_idx_map
            .get(osmid)
            .copied()
            .ok_or_else(|| invalid_data(format!("unknown node id: {osmid}")))
    }

    pub fn node(&self, index: NodeIndex) -> &NodeData {
        &self.graph[index]
    }
}

pub fn read_graph(
    edgelist_path: impl AsRef<Path>,
    nodes_path: impl AsRef<Path>,
) -> GraphResult<GraphContext> {

    let mut graph = EdgeListGraph::new();

    let osmid_idx_map = read_nodes(nodes_path, &mut graph)?;
    read_edges(edgelist_path, &mut graph, &osmid_idx_map)?;

    Ok(GraphContext { graph, osmid_idx_map })
}

fn read_nodes(
    path: impl AsRef<Path>,
    graph: &mut EdgeListGraph,
) -> GraphResult<HashMap<String, NodeIndex>> {

    let mut nodes = HashMap::<String, NodeIndex>::new();

    let mut reader = csv::Reader::from_path(path)?;
    for record in reader.records() {

        let record = record?;
        let osmid = record
            .get(0)
            .ok_or_else(|| invalid_data(format!("missing node osmid in row: {record:?}")))?;
        let lat = parse_csv_float(&record, 1, "lat")?;
        let lon = parse_csv_float(&record, 2, "lon")?;

        let index = graph.add_node(NodeData {
            osmid: osmid.to_owned(),
            lat,
            lon,
        });
        nodes.insert(osmid.to_owned(), index);
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

        let source_idx = find_node(nodes, source)?;
        let target_idx = find_node(nodes, target)?;
        let travel_time = parse_travel_time(attributes)?;

        graph.add_edge(source_idx, target_idx, travel_time);
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
        .trim_start_matches(|c: char| c == '{' || c == '[' || c == '"' || c == '\'')
        .split(|c: char| c == ',' || c == '}' || c == ']')
        .next()
        .unwrap_or("")
        .trim();

    value
        .parse::<TravelTime>()
        .map_err(|err| invalid_data(format!("invalid travelTime value {value:?}: {err}")))
}

fn invalid_data(message: String) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}
