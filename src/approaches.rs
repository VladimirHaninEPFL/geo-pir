

pub fn parse_approach(name: &str) -> Approach<'_> {
    if name.contains("node") {
        Approach {
            name,
            is_node_approach: true,
            block_width: 0.0,
        }
    } else {
        Approach {
            name,
            is_node_approach: false,
            block_width: name
                .trim_start_matches(|c: char| !c.is_ascii_digit())
                .parse()
                .unwrap_or(0.0),
        }
    }
}

pub struct Approach <'a> {
    pub name: &'a str,

    pub is_node_approach: bool,

    pub block_width: f32,
}