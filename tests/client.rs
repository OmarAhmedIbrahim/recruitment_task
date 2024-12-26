use embedded_recruitment_task::message::{client_message, ServerMessage};
use log::info;
use prost::Message;
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

pub struct Client {
    ip: String,
    port: u32,
    timeout: Duration,
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new(ip: &str, port: u32, timeout_ms: u64) -> Self {
        Client {
            ip: ip.to_string(),
            port,
            timeout: Duration::from_millis(timeout_ms),
            stream: None,
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        info!("Connecting to {}:{}", self.ip, self.port);
        let address = format!("{}:{}", self.ip, self.port);
        let socket_addrs: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        if socket_addrs.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid address"));
        }

        let stream = TcpStream::connect_timeout(&socket_addrs[0], self.timeout)?;
        self.stream = Some(stream);
        info!("Connected to the server!");
        Ok(())
    }

    pub fn disconnect(&mut self) -> io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.shutdown(std::net::Shutdown::Both)?;
            info!("Disconnected from the server!");
            self.stream = None;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }

 
    pub fn send(&mut self, message: client_message::Message) -> io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            let mut buffer = Vec::new();
            message.encode(&mut buffer);
            stream.write_all(&buffer)?;
            stream.flush()?;
            info!("Client sending message: {:?}", message); // Log when a message is sent
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotConnected, "No active connection"))
        }
    }

 

    pub fn receive(&mut self) -> io::Result<ServerMessage> {
        if let Some(ref mut stream) = self.stream {
            let mut buffer = vec![0u8; 512];
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read == 0 {
                info!("Server disconnected.");
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Server disconnected",
                ));
            }

            let response = ServerMessage::decode(&buffer[..bytes_read])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Decoding error: {}", e)))?;
            info!("Client received response: {:?}", response); // Log when a response is received
            Ok(response)
        } else {
            Err(io::Error::new(io::ErrorKind::NotConnected, "No active connection"))
        }
    }
}
