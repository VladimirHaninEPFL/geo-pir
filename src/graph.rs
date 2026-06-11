use petgraph::graph::{Graph, NodeIndex};
use petgraph::Directed;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::path::Path;

pub type TravelTimeEdge = u16; // travel time in seconds used for edge weights (2 bytes)
pub type GraphResult<T> = Result<T, Box<dyn Error>>;
pub type EdgeListGraph = Graph<NodeData, TravelTimeEdge, Directed>;

#[derive(Debug, Clone)]
pub struct NodeData {
    // pub osmid: String, // I think we don't need this for the node appraoch
    pub lat: f32,
    pub lon: f32,
}

pub struct GraphContext {
    pub graph: EdgeListGraph,
    pub osmid_idx_map: HashMap<String, NodeIndex>, // here we map the OSM node IDs to their corresponding NodeIndex in the graph for easy lookup when reading edges
    pub idx_osmid_map: HashMap<NodeIndex, String>, // here we map the graph nodeindex to their corresponding osmid. This is used for debugging
}

pub fn read_graph(
    edgelist_path: impl AsRef<Path>,
    nodes_path: impl AsRef<Path>,
) -> GraphResult<GraphContext> {

    let mut graph = EdgeListGraph::new();

    let (osmid_idx_map, idx_osmid_map) = read_nodes(nodes_path, &mut graph)?;
    read_edges(edgelist_path, &mut graph, &osmid_idx_map)?;

    Ok(GraphContext { graph, osmid_idx_map, idx_osmid_map })
}

fn read_nodes(
    path: impl AsRef<Path>,
    graph: &mut EdgeListGraph,
) -> GraphResult<(HashMap<String, NodeIndex>, HashMap<NodeIndex, String>)> {

    // --- 1. Read all rows into a buffer first ---
    struct RawNode {
        osmid: String,
        lat: f32,
        lon: f32,
    }
    let mut raw_nodes: Vec<RawNode> = Vec::new();

    let mut reader = csv::Reader::from_path(path)?;
    for record in reader.records() {
        let record = record?;
        let osmid = record
            .get(0)
            .ok_or_else(|| invalid_data(format!("missing node osmid in row: {record:?}")))?
            .to_owned();
        let lat = parse_csv_float(&record, 1, "lat")?;
        let lon = parse_csv_float(&record, 2, "lon")?;
        raw_nodes.push(RawNode { osmid, lat, lon });
    }

    // --- 2. Sort by Hilbert-curve index for spatial locality ---
    //
    // We map (lat, lon) into a [0, 2^16) integer grid, then interleave
    // the bits of (row, col) to get a Hilbert index. Points that are
    // close in 2-D end up with close indices, so graph node indices
    // reflect geographic proximity.
    const ORDER: u32 = 16;              // 2^16 × 2^16 grid
    const N: f32 = (1u32 << ORDER) as f32; // 65536.0

    // Bounding box — needed to normalise coordinates into [0, N)
    let min_lat = raw_nodes.iter().map(|n| n.lat).fold(f32::INFINITY,  f32::min);
    let max_lat = raw_nodes.iter().map(|n| n.lat).fold(f32::NEG_INFINITY, f32::max);
    let min_lon = raw_nodes.iter().map(|n| n.lon).fold(f32::INFINITY,  f32::min);
    let max_lon = raw_nodes.iter().map(|n| n.lon).fold(f32::NEG_INFINITY, f32::max);

    let lat_range = (max_lat - min_lat).max(f32::EPSILON);
    let lon_range = (max_lon - min_lon).max(f32::EPSILON);

    /// Convert a (row, col) grid position into a Hilbert curve index.
    /// Classic in-place rotation algorithm; O(ORDER) bit operations.
    fn hilbert_index(mut row: u32, mut col: u32, order: u32) -> u64 {
        let mut index: u64 = 0;
        let mut level = 1u32 << (order - 1);
        while level > 0 {
            let rx = if (row & level) > 0 { 1u64 } else { 0 };
            let ry = if (col & level) > 0 { 1u64 } else { 0 };
            index += (level as u64).pow(2) * ((3 * rx) ^ ry);
            // Rotate / reflect the quadrant
            if ry == 0 {
                if rx == 1 {
                    row = (1 << order) - 1 - row;
                    col = (1 << order) - 1 - col;
                }
                std::mem::swap(&mut row, &mut col);
            }
            level >>= 1;
        }
        index
    }

    raw_nodes.sort_unstable_by_key(|n| {
        let r = ((n.lat - min_lat) / lat_range * (N - 1.0)) as u32;
        let c = ((n.lon - min_lon) / lon_range * (N - 1.0)) as u32;
        hilbert_index(r, c, ORDER)
    });

    // --- 3. Insert into the graph in Hilbert order ---
    let mut osmid_idx_map = HashMap::<String, NodeIndex>::new();
    let mut idx_osmid_map = HashMap::<NodeIndex, String>::new();

    for node in raw_nodes {
        let index = graph.add_node(NodeData { lat: node.lat, lon: node.lon });
        osmid_idx_map.insert(node.osmid.clone(), index);
        idx_osmid_map.insert(index, node.osmid.clone());
    }

    Ok((osmid_idx_map, idx_osmid_map))
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
        let travel_time = parse_travel_time(attributes)?.round() as TravelTimeEdge;

        graph.add_edge(source_idx, target_idx, travel_time);
    }

    Ok(())
}

fn parse_csv_float(record: &csv::StringRecord, index: usize, name: &str) -> io::Result<f32> {
    let value = record
        .get(index)
        .ok_or_else(|| invalid_data(format!("missing {name} in row: {record:?}")))?;

    value
        .parse::<f32>()
        .map_err(|err| invalid_data(format!("invalid {name} value {value:?}: {err}")))
}

fn find_node(nodes: &HashMap<String, NodeIndex>, id: &str) -> io::Result<NodeIndex> {
    nodes
        .get(id)
        .copied()
        .ok_or_else(|| invalid_data(format!("edge references unknown node id: {id}")))
}

fn parse_travel_time(attributes: &str) -> io::Result<f64> {
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
        .parse::<f64>()
        .map_err(|err| invalid_data(format!("invalid travelTime value {value:?}: {err}")))
}

fn invalid_data(message: String) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}
