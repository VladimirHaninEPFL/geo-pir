use std::env;

use geo_pir::{db_settings::Countries, graph::{GraphContext, GraphResult}};

// todo: genrate a graph context if not already created, no need to execute it

fn main() -> GraphResult<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!(
            "Usage: {} <country_name>",
            args[0]
        );
        std::process::exit(1);
    }

    let country_name = &args[1];
    let country = country_name
        .parse::<Countries>()
        .expect("unknown country name");

    let graph_context = GraphContext::new(&country)?;
    graph_context.save(&country)?;

    Ok(())
}
