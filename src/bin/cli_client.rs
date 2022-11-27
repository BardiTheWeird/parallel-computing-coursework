use std::{net::TcpStream, io};

use clap::{Parser, ValueEnum};
use log::{info, debug, warn, error};
use parallel_computing::messages::{Request, IntoMessage, Response, FromMessage};

#[derive(Parser, Debug)]
struct Arguments {
    #[arg(short = 's', long = "server-address", default_value = "127.0.0.1:8080")]
    server_address: String,

    #[arg(short = 'r', long = "request-kind", default_value = "ping")]
    request_kind: RequestKindCli,

    #[arg(short = 'p', long = "payload")]
    payload: Option<String>
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum RequestKindCli {
    Ping,
    Index,
    File,
}

fn main() -> io::Result<()> {
    env_logger::init();

    let arguments = Arguments::parse();
    debug!("{:?}", arguments);

    let request = match (arguments.request_kind, arguments.payload) {
        (RequestKindCli::Ping, x) => {
            if x.is_some() {
                warn!("ping request does not require a payload")
            }
            Request::Ping
        },
        (RequestKindCli::Index, Some(query)) => 
            Request::Query(query.to_string()),
        (RequestKindCli::File, Some(filepath)) => 
            Request::QueryFile(filepath.to_string()),

        (request_kind, None) => {
            error!("{:?} request requires a payload", request_kind);
            std::process::exit(1);
        }
    };

    info!("connecting to a server at {}...", arguments.server_address);
    let mut stream = TcpStream::connect(arguments.server_address)?;
    
    request.write(&mut stream)?;

    let response = Response::read(&mut stream)?;
    if let Response::QueryResult(res) = response {
        println!("{}", serde_yaml::to_string(&res).unwrap());
    } else {
        println!("{:?}", response);
    }

    Ok(())
}