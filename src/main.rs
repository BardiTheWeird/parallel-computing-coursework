mod word_filtering;
mod inverted_index;
mod word_stemming;

use std::{fs::{File}, path::{Path, PathBuf}, io, collections::HashSet};

use clap::{Command, Arg, ArgAction};
use inverted_index::InvertedIndex;
use log::error;
use word_filtering::reader_to_words;


fn main() {
    env_logger::init();

    let mut inverted_index = InvertedIndex::new();

    let matches = Command::new("Someone's App")
        .arg(
            Arg::new("directory")
                .short('d')
                .required(false)
                .action(ArgAction::Append)
        ).get_matches();
    
    if let Some(directories) = matches.get_many::<String>("directory") {
        let directories = directories
            .map(|s| Path::new(s))
            .filter(|p| {
                if !p.exists() {
                    error!("{:?} does not exist", p);
                    return false;
                }
                if !p.is_dir() {
                    error!("{:?} is not a directory", p);
                    return false;
                }
                true
            });

        let files = get_file_paths_from_directories(directories);
        insert_files_into_inverted_index(files, &mut inverted_index);
    }
}

fn insert_files_into_inverted_index(files: Vec<PathBuf>, inverted_index: &mut InvertedIndex) {
    for file_path in files {
        match unique_words_in_file(&file_path) {
            Ok(words) => inverted_index.insert(file_path, words),
            Err(err) => error!("Error reading unique words in {:?};
            error: {}", file_path, err),
        }
    }
}

fn unique_words_in_file(file_path: &PathBuf) -> io::Result<HashSet<String>> {
    let file_handle = File::open(file_path)?;
    reader_to_words(file_handle)
}

fn get_file_paths_from_directories<'a>(directory_paths: impl Iterator<Item = &'a Path>) -> Vec<PathBuf> {
    directory_paths.flat_map(|p| -> Box<dyn Iterator<Item = PathBuf>> {
        if let Ok(dir_iterator) = p.read_dir() {
            Box::new(dir_iterator
            .filter_map(|dir_entry| {
                if let Ok(f) = dir_entry{
                    let file_path = f.path();
                    if file_path.is_file() {
                        return Some(file_path);
                    }
                }
                return None
            }))
        } else {
            Box::new(std::iter::empty())
        }
    }).collect()
}
