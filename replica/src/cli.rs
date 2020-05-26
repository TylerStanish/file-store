use clap::{Arg, ArgMatches, App, SubCommand};
use crate::index::LocalIndex;

pub fn run_cli() {
    let matches = parse_args();
    if let Some(mut matches) = matches.subcommand_matches("index").and_then(|matches| matches.values_of("directories")) {
        let mut local_index = LocalIndex::from_local();
        while let Some(dir) = matches.next() {
            let tag_name = dir;
            let root_path = matches.next().unwrap(); // clap should guarantee each occurrence has 2 values
            local_index.new_tag(tag_name, root_path);
        }
        local_index.persist_local();
        //println!("{:?}", local_index.redundancies());
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("Local redundancy indexer")
        .subcommand(SubCommand::with_name("index")
            .arg(Arg::with_name("directories")
                .short("t")
                .multiple(true)
                .number_of_values(2)
                .takes_value(true)
                .help("A parent directory to index"))
            .help("Index the directories"))
        .get_matches()
}
