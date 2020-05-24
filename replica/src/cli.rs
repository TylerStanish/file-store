use clap::{Arg, ArgMatches, App, SubCommand};
use crate::index::LocalIndex;

pub fn run_cli() {
    let matches = parse_args();
    if let Some(matches) = matches.subcommand_matches("index").and_then(|matches| matches.values_of("directories")) {
        let mut local_index = LocalIndex::new();
        for dir in matches {
            local_index.index(dir);
        }
        println!("{}", local_index.to_json())
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
