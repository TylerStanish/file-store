use std::path::Path;
use clap::{Arg, App, SubCommand};
use crate::index;

pub fn run_cli() {
    let args = parse_args();
}

fn parse_args() {
    let matches = App::new("Local redundancy indexer")
        .subcommand(SubCommand::with_name("index")
            .arg(Arg::with_name("directories")
                 .multiple(true))
            .help("Index the directories"))
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("index").and_then(|matches| matches.values_of("directories")) {
        for dir in matches {
            index::hash(&Path::new(dir));
        }
    }
}
