use std::{sync::Arc, time::Instant};

use clap::{Command, Arg, ArgAction};
use log::{info, error};
use parallel_computing::{inverted_index::InvertedIndex, fs_helpers, server::Server};

fn main() {
    env_logger::init();

    let matches = Command::new("Someone's App")
        .arg(
            Arg::new("directory")
                .short('d')
                .required(false)
                .action(ArgAction::Append)
        ).arg(
            Arg::new("server-address")
                .short('s')
                .default_value("127.0.0.1:8080")
        ).arg(
            Arg::new("thread-count")
                .short('t')
                .default_value("1")
        )
        .get_matches();
    
    let thread_count = matches.get_one::<String>("thread-count").unwrap();
    let thread_count = match usize::from_str_radix(&thread_count, 10) {
        Ok(x) => x,
        Err(err) => {
            error!("error parsing thread_count: {:?}", err);
            std::process::exit(1);
        },
    };
    assert!(thread_count > 0, "thread count can't be less than 1");
    let inverted_index = Arc::new(InvertedIndex::new());
    if let Some(directories) = matches.get_many::<String>("directory") {
        info!("Constructing index from files in provided directories");
        let files = fs_helpers::get_file_paths_from_directories(directories.map(|s| s));
        info!("{} files found", files.len());
        let index_construction_start = Instant::now();
        fs_helpers::insert_files_into_inverted_index(Arc::new(files), &inverted_index, thread_count);
        info!("index construction took {}ms", index_construction_start.elapsed().as_millis());
    }

    let addr = matches.get_one::<String>("server-address").unwrap();
    info!("serving at {}...", addr);

    let mut server = Server::new(inverted_index, thread_count);
    if let Err(err) = server.listen(addr) {
        error!("critical server error: {}", err);
    }
}
