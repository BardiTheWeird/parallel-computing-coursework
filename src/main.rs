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
        ).get_matches();
    
    let mut inverted_index = InvertedIndex::new();
    if let Some(directories) = matches.get_many::<String>("directory") {
        info!("Constructing index from files in provided directories");
        let files = fs_helpers::get_file_paths_from_directories(directories.map(|s| s));
        info!("{} files found", files.len());
        fs_helpers::insert_files_into_inverted_index(files, &mut inverted_index);
        info!("index constructed");
    }

    let mut server = Server::new(inverted_index);
    if let Err(err) = server.listen("127.0.0.1:8080") {
        error!("critical server error: {}", err);
    }
}
