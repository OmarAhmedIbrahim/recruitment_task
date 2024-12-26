use log::info;
use std::{process, sync::Arc};
use embedded_recruitment_task::server::Server;

fn main() {
    // Initialize logging
    env_logger::init();

    let server_address = "127.0.0.1:8080";
    let num_threads = 4;

    // Create the server
    let server = match Server::new(server_address, num_threads) {
        Ok(server) => Arc::new(server),
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            process::exit(1);
        }
    };

    // Run the server in a separate thread
    let server_arc = Arc::clone(&server);
    let server_handle = std::thread::spawn(move || {
        if let Err(e) = server_arc.run() {
            eprintln!("Server encountered an error: {}", e);
        }
    });

    // Handle graceful shutdown on Ctrl+C
    ctrlc::set_handler({
        let server = Arc::clone(&server);
        move || {
            info!("Shutdown signal received.");
            server.stop();
        }
    })
    .expect("Failed to set Ctrl+C handler");

    // Wait for the server thread to finish
    if let Err(e) = server_handle.join() {
        eprintln!("Error waiting for server thread to finish: {:?}", e);
    }

    info!("Server has shut down.");
}
