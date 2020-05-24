use std::collections::HashSet;
use clap::{Arg, ArgMatches, App, SubCommand};
use crate::index::LocalIndex;

pub fn run_cli() {
    let matches = parse_args();
    if let Some(matches) = matches.subcommand_matches("index").and_then(|matches| matches.values_of("directories")) {
        let mut local_index = LocalIndex::from_local();
        for dir in matches {
            local_index.index(dir);
        }
        local_index.persist_local();
        println!("{:?}", local_index.redundancies());
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("Local redundancy indexer")
        .subcommand(SubCommand::with_name("index")
            .arg(Arg::with_name("directories")
                 .multiple(true))
            .help("Index the directories"))
        .get_matches()
}
