use std::{net::TcpStream, io};

use parallel_computing::messages::{Request, IntoMessage, Response, FromMessage};

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let request = Request::Query("Frank Sinatra".to_owned());
    request.write(&mut stream)?;
    let response = Response::read(&mut stream)?;
    println!("{:?}", response);

    Ok(())
}