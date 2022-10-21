use std::{net::TcpStream, io};

use clap::{Command, Arg};
use log::{info, debug};
use parallel_computing::messages::{Request, IntoMessage, Response, FromMessage};

fn main() -> io::Result<()> {
    env_logger::init();

    let matches = Command::new("CLI Client")
        .arg(Arg::new("server-address")
            .short('s')
            .default_value("127.0.0.1:8080")
        ).get_matches();

    let addr = matches.get_one::<String>("server-address").unwrap();
    info!("connecting to a server at {}...", addr);

    let mut stream = TcpStream::connect(addr)?;
    let request = Request::Query("Frank Sinatra".to_owned());
    request.write(&mut stream)?;
    let response = Response::read(&mut stream)?;
    println!("{:?}", response);

    Ok(())
}