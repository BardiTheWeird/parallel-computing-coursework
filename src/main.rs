use std::{sync::Arc, time::Instant, num::NonZeroUsize};

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use log::{info, error, debug};

use parallel_computing::{inverted_index::InvertedIndex, fs_helpers, server::Server};
use serde::Serialize;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Time {
        #[arg(short = 'd', long = "directory", action = ArgAction::Append)]
        directories: Vec<String>,

        #[arg(long = "thread-start")]
        thread_count_start: NonZeroUsize,

        #[arg(long = "thread-end")]
        thread_count_end: NonZeroUsize,

        #[arg(short = 'o', default_value = "json")]
        output_format: OutputFormat,

        #[arg(short = 'i', default_value = "10")]
        iterations: NonZeroUsize,
    },
    Serve {
        #[arg(short = 's', long = "server-address", default_value = "127.0.0.1:8080")]
        server_address: String,

        #[arg(short = 'd', long = "directory", action = ArgAction::Append)]
        directories: Option<Vec<String>>,

        #[arg(short = 't', long = "thread-count", default_value = "1")]
        thread_count: NonZeroUsize,
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    Json,
    Yaml,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    debug!("{:?}", cli);

    match cli.commands {
        Commands::Time {
            directories,
            thread_count_start,
            thread_count_end,
            output_format,
            iterations,
        } =>  {
            if thread_count_end < thread_count_start {
                eprintln!("thread-start should be less than or equal to thread-end");
                std::process::exit(1)
            }

            let thread_count_start = usize::from(thread_count_start);
            let thread_count_end = usize::from(thread_count_end);
            let iterations = usize::from(iterations);

            let files = fs_helpers::get_file_paths_from_directories(directories.iter());
            let files = Arc::new(files);
            eprintln!("{} files found", files.len());

            let time = | thread_count | {
                let index_construction_start = Instant::now();

                for _ in 0..iterations {
                    let inverted_index = Arc::new(InvertedIndex::new());
                    fs_helpers::insert_files_into_inverted_index(Arc::clone(&files), &inverted_index, thread_count);
                }

                return index_construction_start.elapsed().as_nanos() / iterations as u128;
            };

            #[derive(Serialize)]
            struct ResultInstance {
                threads: usize,
                time: u128,
            }

            let results : Vec<ResultInstance> = (thread_count_start..thread_count_end+1)
                .map(|thread_count| ResultInstance {
                    threads: thread_count,
                    time: time(thread_count)
                }).collect();

            let results = match output_format {
                OutputFormat::Json => serde_json::to_string(&results).unwrap(),
                OutputFormat::Yaml => serde_yaml::to_string(&results).unwrap(),
            };
            println!("{}", results);
        },
        Commands::Serve {
            server_address,
            directories,
            thread_count,
        } => {
            let thread_count = usize::from(thread_count);

            let inverted_index = Arc::new(InvertedIndex::new());
            if let Some(directories) = directories {
                info!("Constructing index from files in provided directories");
                let files = fs_helpers::get_file_paths_from_directories(directories.iter());
                fs_helpers::insert_files_into_inverted_index(Arc::new(files), &inverted_index, thread_count);
            }
        
            info!("serving at {}...", server_address);
            let mut server = Server::new(inverted_index, thread_count);
            if let Err(err) = server.listen(server_address) {
                error!("critical server error: {}", err);
            }
        },
    }
}
