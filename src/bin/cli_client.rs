use std::{net::TcpStream, io};

use clap::{Command, Arg};
use log::{info, debug};
use parallel_computing::messages::{Request, IntoMessage, Response, FromMessage};

fn main() -> io::Result<()> {
    env_logger::init();

    let matches = Command::new("CLI Client")
        .arg(Arg::new("server-address")
            .short('s')
            .default_value("127.0.0.1:8080"))
        .arg(Arg::new("request-kind")
            .short('r')
            .default_value("index"))
        .arg(Arg::new("query")
            .required(true))
        .get_matches();

    let request_kind = matches.get_one::<String>("request-kind").unwrap();
    let query = matches.get_one::<String>("query").unwrap();

    let request = match request_kind.as_str() {
        "index" => Request::Query(query.to_string()),
        "file" => Request::QueryFile(query.to_string()),
        x => return Err(io::Error::new(io::ErrorKind::InvalidInput, 
            format!("{} is not a request kind", x)))
    };

    let addr = matches.get_one::<String>("server-address").unwrap();
    
    info!("connecting to a server at {}...", addr);
    let mut stream = TcpStream::connect(addr)?;
    
    info!("querying for `{}`...", &query);
    request.write(&mut stream)?;

    let response = Response::read(&mut stream)?;
    if let Response::QueryResult(res) = response {
        println!("{}", serde_yaml::to_string(&res).unwrap());
    } else {
        println!("{:?}", response);
    }

    Ok(())
}