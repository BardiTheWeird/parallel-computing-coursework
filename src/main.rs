mod word_filtering;
mod inverted_index;
mod word_stemming;
mod fs_helpers;

use clap::{Command, Arg, ArgAction};
use inverted_index::InvertedIndex;
use log::info;

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
}
