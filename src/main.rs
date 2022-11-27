use std::{sync::Arc, time::Instant, num::NonZeroUsize};

use clap::{ArgAction, Parser};
use log::{info, error, debug};

use parallel_computing::{inverted_index::InvertedIndex, fs_helpers, server::Server};

#[derive(Parser, Debug)]
struct Arguments {
    #[arg(short = 's', long = "server-address", default_value = "127.0.0.1:8080")]
    server_address: String,

    #[arg(short = 'd', long = "directory", action = ArgAction::Append)]
    directories: Vec<String>,

    #[arg(short = 't', long = "thread-count", default_value = "1")]
    thread_count: NonZeroUsize,
}

fn main() {
    env_logger::init();

    let arguments = Arguments::parse();
    let thread_count = usize::from(arguments.thread_count);
    debug!("{:?}", arguments);

    let inverted_index = Arc::new(InvertedIndex::new());
    if !arguments.directories.is_empty() {
        info!("Constructing index from files in provided directories");
        let files = fs_helpers::get_file_paths_from_directories(arguments.directories.iter());
        info!("{} files found", files.len());
        let index_construction_start = Instant::now();
        fs_helpers::insert_files_into_inverted_index(Arc::new(files), &inverted_index, thread_count);
        info!("index construction took {}ms", index_construction_start.elapsed().as_millis());
    }

    info!("serving at {}...", arguments.server_address);
    let mut server = Server::new(inverted_index, thread_count);
    if let Err(err) = server.listen(arguments.server_address) {
        error!("critical server error: {}", err);
    }
}
