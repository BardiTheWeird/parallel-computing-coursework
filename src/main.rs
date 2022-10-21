use clap::{Command, Arg, ArgAction};
use log::{info, error, debug};
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
        ).get_matches();
    
    let mut inverted_index = InvertedIndex::new();
    if let Some(directories) = matches.get_many::<String>("directory") {
        info!("Constructing index from files in provided directories");
        let files = fs_helpers::get_file_paths_from_directories(directories.map(|s| s));
        info!("{} files found", files.len());
        fs_helpers::insert_files_into_inverted_index(files, &mut inverted_index);
        info!("index constructed");
    }

    let addr = matches.get_one::<String>("server-address").unwrap();
    info!("serving at {}...", addr);

    let mut server = Server::new(inverted_index);
    if let Err(err) = server.listen(addr) {
        error!("critical server error: {}", err);
    }
}
