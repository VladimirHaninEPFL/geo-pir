use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct OutgoingEdge {
    pub id_target: u32, // this represents the graph id of the neighbour
    pub cost: u16,
    pub _pad: u16, // explicit padding so that the struct is aligned
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node0Entry {
    pub latitude: f32,
    pub longitude: f32,
    pub outgoing_edges: [OutgoingEdge; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node1Entry {
    node0_entry: Node0Entry,
    neighbours: [Node0Entry; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node2Entry {
    node0_entry: Node0Entry,
    neighbours: [Node1Entry; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Node3Entry {
    node0_entry: Node0Entry,
    neighbours: [Node2Entry; 4],
}

pub struct LogicalDatabase {
    pub num_records: usize,
    pub record_size_bytes: usize,
}

pub fn get_logical_db(country_name: &str, approach: &str) -> LogicalDatabase {
    
    // todo: finish this table !
    if country_name == "Switzerland" {
        if approach == "node0" {
            return LogicalDatabase {
                num_records: 467_344,
                record_size_bytes: std::mem::size_of::<Node0Entry>()
            };
        } else { // this is wrong
            return LogicalDatabase {
                num_records: 467_344,
                record_size_bytes: std::mem::size_of::<Node0Entry>()
            };
        }
    }
    else if country_name == "France" {
        if approach == "node0" {
            return LogicalDatabase {
                num_records: 5_196_479,
                record_size_bytes: std::mem::size_of::<Node0Entry>()
            };
        }
        else { // this is wrong
            return LogicalDatabase {
                num_records: 5_196_479,
                record_size_bytes: std::mem::size_of::<Node0Entry>()
            };
        }
    }

    else {
        panic!("didn't define parameters for that country");
    }
}

