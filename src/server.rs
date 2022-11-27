use std::{net::{ToSocketAddrs, TcpListener, TcpStream}, io, time::Duration, sync::{Arc, mpsc, Mutex}, thread};

use log::{error};

use crate::{inverted_index::InvertedIndex, messages::{Request, Response, FromMessage, IntoMessage}};

pub struct Server {
    inverted_index: Arc<InvertedIndex>,
    thread_pool: ThreadPool,
}

impl Server {
    pub fn new(inverted_index: Arc<InvertedIndex>, thread_count: usize) -> Self {
        Self { inverted_index, thread_pool: ThreadPool::new(thread_count) }
    }

    pub fn listen(&mut self, addr: impl ToSocketAddrs) -> io::Result<()> {
        let listeners = TcpListener::bind(addr)?;
        for stream in listeners.incoming() {
            match stream {
                Ok(mut x) => {
                    let inverted_index = Arc::clone(&self.inverted_index);
                    self.thread_pool.run_job(move ||{
                        if let Err(err) = Self::handle_stream(&mut x, inverted_index) {
                            error!("Connection to {:?} ended with an error: {}", 
                                x.peer_addr(), err);
                        }
                    });
                },
                Err(err) => {
                    error!("{}", err);
                    continue;
                },
            };
        }
        Ok(())
    }

    fn handle_stream(stream: &mut TcpStream, inverted_index: Arc<InvertedIndex>) -> io::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;

        let request = Request::read(stream)?;
        let response = match request {
            Request::Ping => Response::Pong,
            Request::Query(s) => Response::QueryResult(
                inverted_index.query(&s)),
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

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    fn run_job<T>(&mut self, job: T)
        where T: FnOnce() + Send + 'static
    {
        if let Some(sender) = &self.sender {
            sender.send(Box::new(job)).unwrap();
        }
    }

    fn new(thread_count: usize) -> Self {
        let mut workers = Vec::with_capacity(thread_count);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..thread_count {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        Self { workers, sender: Some(sender) }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            }
        });
        Self { thread: Some(thread) }
    }
}
