use std::{collections::HashMap, fmt, str::FromStr};
use serde::{Serialize, Deserialize};
use crate::{data_entries::*, graph::EdgeListGraph};


// this struct sotres the parameters of the database, taht the server generates,
// and that the client retrieves as it needs them
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBSettings {
    pub approach: Approaches,
    pub country: Countries,
    pub architecture: Architectures,

    pub block_params: Option<BlockParams>,
    pub logical_db: LogicalDatabase,
}

impl DBSettings {

    pub fn new(country_name: &str, architecture_name: &str, approach_name: &str, graph: &EdgeListGraph) -> Self {
        println!("db settings creation...");

        let country = country_name
            .parse::<Countries>()
            .expect("unknown country name");

        let architecture = architecture_name
            .parse::<Architectures>()
            .expect("unknown country name");

        let approach = approach_name
            .parse::<Approaches>()
            .expect("unknown approach name");

        let block_params: Option<BlockParams> = match approach {
            Approaches::Block(block_width) => Some(BlockParams::new(graph, block_width)),
            _ => None,
        };

        let logical_db = LogicalDatabase::new(&approach, graph, &block_params);

        DBSettings { approach, country, architecture, block_params, logical_db}
    }

    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("serialization failed")
    }

    pub fn deserialize_from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("deserialization failed")
    }
    
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockParams {
    pub nodeidx_blockid_map: HashMap<u32, BlockId>,
    pub num_blocks: usize,
    pub nodes_per_block: usize,
}

impl BlockParams {

    pub fn new(graph: &EdgeListGraph, block_width: f32) -> Self {

        let mut nodeidx_blockid_map: HashMap<u32, BlockId> = HashMap::new();

        let mut block_id_by_cell: HashMap<(i32, i32), BlockId> = HashMap::new();
        let mut next_block_id: BlockId = 0;

        let mut block_node_counts: HashMap<BlockId, usize> = HashMap::new();

        for node_idx in graph.node_indices() {

            let node_data = &graph[node_idx];

            let cell_row = (node_data.lat / block_width).floor() as i32;
            let cell_col = (node_data.lon / block_width).floor() as i32;
            let cell = (cell_row, cell_col);

            let block_id = *block_id_by_cell.entry(cell).or_insert_with(|| {
                let id = next_block_id;
                next_block_id += 1;
                id
            });
            nodeidx_blockid_map.insert(node_idx.index() as u32, block_id);

            *block_node_counts.entry(block_id).or_insert(0) += 1;
        }

        let max_nodes_in_block = block_node_counts.values().copied().max().unwrap_or(0);

        BlockParams {
            nodeidx_blockid_map,
            num_blocks: next_block_id,
            nodes_per_block: max_nodes_in_block,
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Countries {
    Switzerland,
    France,
}

impl FromStr for Countries {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {

            "Switzerland" => Ok(Countries::Switzerland),
            "France" => Ok(Countries::France),

            _ => Err(format!("unknown country: {s}")),
        }
    }
}
impl fmt::Display for Countries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Countries::Switzerland   => write!(f, "Switzerland"),
            Countries::France   => write!(f, "France"),
        }
    }
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Architectures {
    Spiral,
    SinglePass,
    Naive,
}

impl FromStr for Architectures {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {

            "Spiral" => Ok(Architectures::Spiral),
            "SinglePass" => Ok(Architectures::SinglePass),
            "Naive" => Ok(Architectures::Naive),

            _ => Err(format!("unknown architecutre: {s}")),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Approaches {
    Node0,
    Node1,
    Node2,
    Node3,
    Block(f32),
}

impl FromStr for Approaches {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {

            "node0" => Ok(Approaches::Node0),
            "node1" => Ok(Approaches::Node1),
            "node2" => Ok(Approaches::Node2),
            "node3" => Ok(Approaches::Node3),

            s if s.starts_with("block") => {

                let block_width: f32 = s
                    .trim_start_matches(|c: char| !c.is_ascii_digit())
                    .replace('_', ".")
                    .parse()
                    .map_err(|e| format!("invalid block width: {e}"))?;

                Ok(Approaches::Block(block_width))
            }
            _ => Err(format!("unknown approach: {s}")),
        }
    }
}
impl fmt::Display for Approaches {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Approaches::Node0   => write!(f, "node0"),
            Approaches::Node1   => write!(f, "node1"),
            Approaches::Node2   => write!(f, "node2"),
            Approaches::Node3   => write!(f, "node3"),
            Approaches::Block(0.1)   => write!(f, "block0.1"),
            Approaches::Block(0.25)   => write!(f, "block0.25"),
            Approaches::Block(0.5)   => write!(f, "block0.5"),
            Approaches::Block(1.)   => write!(f, "block1"),
            _   => write!(f, "block1"),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalDatabase {
    pub num_records: usize,
    pub record_size_bytes: usize,
}

impl LogicalDatabase {

    pub fn new(approach: &Approaches, graph: &EdgeListGraph, block_params: &Option<BlockParams>) -> LogicalDatabase {
    
        match approach {
            Approaches::Node0 => LogicalDatabase {
                                    num_records: graph.node_count(),
                                    record_size_bytes: std::mem::size_of::<Node0Entry>()
                                },
            Approaches::Node1 => LogicalDatabase {
                                    num_records: graph.node_count(),
                                    record_size_bytes: std::mem::size_of::<Node1Entry>()
                                },
            Approaches::Node2 => LogicalDatabase {
                                    num_records: graph.node_count(),
                                    record_size_bytes: std::mem::size_of::<Node2Entry>()
                                },
            Approaches::Node3 => LogicalDatabase {
                                    num_records: graph.node_count(),
                                    record_size_bytes: std::mem::size_of::<Node3Entry>()
                                },

            Approaches::Block(_) => LogicalDatabase {
                                        num_records: block_params.as_ref().unwrap().num_blocks,
                                        record_size_bytes: block_params.as_ref().unwrap().nodes_per_block * std::mem::size_of::<BlockEntry>(),
                                    },
        }
    }

}
