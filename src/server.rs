use std::{net::{ToSocketAddrs, TcpListener, TcpStream}, io, time::Duration};

use log::{error};

use crate::{inverted_index::InvertedIndex, messages::{Request, Response, FromMessage, IntoMessage}};

pub struct Server {
    inverted_index: InvertedIndex
}

impl Server {
    pub fn new(inverted_index: InvertedIndex) -> Self {
        Self { inverted_index }
    }

    pub fn listen(&mut self, addr: impl ToSocketAddrs) -> io::Result<()> {
        let listeners = TcpListener::bind(addr)?;
        for stream in listeners.incoming() {
            match stream {
                Ok(mut x) => {
                    if let Err(err) = self.handle_stream(&mut x) {
                        error!("Connection to {:?} ended with an error: {}", 
                            x.peer_addr(), err);
                    }
                },
                Err(err) => {
                    error!("{}", err);
                    continue;
                },
            };
        }
        Ok(())
    }

    fn handle_stream(&mut self, stream: &mut TcpStream) -> io::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;

        let request = Request::read(stream)?;
        let response = match request {
            Request::Ping => Response::Pong,
            Request::Query(s) => Response::QueryResult(
                self.inverted_index.query(&s)),
            Request::QueryFile(s) => {
                match Response::from_file_path(&s) {
                    Ok(r) => r,
                    Err(err) => {
                        error!("error opening file {}: {:?}", &s, err);
                        Response::Error("Error opening file".to_owned())
                    },
                }
                
            },
        };
        response.write(stream)
    }
}
