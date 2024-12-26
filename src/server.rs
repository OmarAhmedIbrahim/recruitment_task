use crate::message::EchoMessage;
use log::{error, info};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use threadpool::ThreadPool;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }
    pub fn handle(mut self) -> io::Result<()> {
        info!("Processing message from client...");
        let mut buffer = [0; 512];
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }

        match EchoMessage::decode(&buffer[..bytes_read]) {
            Ok(message) => {
                info!("Received: {}", message.content);
                let payload = message.encode_to_vec();
                self.stream.write_all(&payload)?;
                self.stream.flush()?;
                info!("Response sent to client.");
            }
            Err(e) => {
                error!("Failed to decode message: {}", e);
            }
        }

        Ok(())
    }
}

   

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
    thread_pool: ThreadPool,
}

impl Server {
    pub fn new(addr: &str, num_threads: usize) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        let thread_pool = ThreadPool::new(num_threads);
        Ok(Server {
            listener,
            is_running,
            thread_pool,
        })
    }

    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    self.thread_pool.execute(move || {
                        let client = Client::new(stream);
                        if let Err(e) = client.handle() {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server shutting down.");
        Ok(())
    }

    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            self.thread_pool.join();
            info!("Server stopped.");
        }
    }
}
