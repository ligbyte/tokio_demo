use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use std::sync::Arc;

// Limit the maximum number of concurrent connections
const MAX_CONNECTIONS: usize = 1000;
// Buffer size for reading client data
const BUFFER_SIZE: usize = 256;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("192.168.1.107:60000").await?;
    
    // Create a semaphore to limit concurrent connections
    let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));

    loop {
        let (mut socket, addr) = listener.accept().await?;
        
        // Clone the semaphore for use in the spawned task
        let semaphore = semaphore.clone();
        
        tokio::spawn(async move {
            // Acquire a permit for this connection
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    eprintln!("Failed to acquire semaphore permit");
                    return;
                }
            };

            // Use stack allocation instead of heap allocation for the buffer
            let mut buf = [0u8; BUFFER_SIZE];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Convert the received bytes to a string and print it
                let received_msg = String::from_utf8_lossy(&buf[..n]);
                println!("Client {:?} sent: {}", addr, received_msg);

                // Pre-allocate string with capacity to reduce reallocations
                // Calculate the needed capacity: original message + "00000000000"
                let mut response_msg = String::with_capacity(received_msg.len() + 11);
                response_msg.push_str(&received_msg);
                response_msg.push_str("00000000000");

                // Write the modified data back to the client
                if let Err(e) = socket.write_all(response_msg.as_bytes()).await {
                    eprintln!("Failed to write to socket; err = {:?}", e);
                    return;
                }
                
                // Ensure the response is sent immediately
                if let Err(e) = socket.flush().await {
                    eprintln!("Failed to flush socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}