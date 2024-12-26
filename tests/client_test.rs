use embedded_recruitment_task::{
    message::{client_message, server_message, EchoMessage},
    server::Server,
};
use std::{sync::Arc, thread::{self, JoinHandle}};

mod client;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = server.run() {
            log::error!("Server encountered an error: {}", e);
        }
    })
}

fn create_server(port: u16) -> Arc<Server> {
    Arc::new(Server::new(&format!("localhost:{}", port), 4).expect("Failed to start server"))
}

#[test]
fn test_client_echo_message() {
    let server = create_server(8082);
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8082, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, Server!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(response.is_ok(), "Failed to receive response");

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    handle.join().expect("Server thread failed to stop");
}

#[test]
fn test_multiple_clients() {
    let server = create_server(8083);
    let handle = setup_server_thread(server.clone());

    let messages = vec![
        "Hello from Client 1".to_string(),
        "Hello from Client 2".to_string(),
        "Hello from Client 3".to_string(),
    ];

    for (i, message_content) in messages.iter().enumerate() {
        let mut client = client::Client::new("localhost", 8083, 1000);
        assert!(client.connect().is_ok(), "Client {} failed to connect", i + 1);

        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        assert!(client.send(message).is_ok(), "Client {} failed to send message", i + 1);

        let response = client.receive();
        assert!(response.is_ok(), "Client {} failed to receive response", i + 1);
        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(echo.content, *message_content, "Response mismatch for Client {}", i + 1);
            }
            _ => panic!("Unexpected response for Client {}", i + 1),
        }

        assert!(client.disconnect().is_ok(), "Client {} failed to disconnect", i + 1);
    }

    server.stop();
    handle.join().expect("Server thread failed to stop");
}
