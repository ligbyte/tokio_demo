use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use std::sync::Arc;

// Limit the maximum number of concurrent connections
const MAX_CONNECTIONS: usize = 1000;

#[tokio::main]
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

            // Use a smaller buffer size to reduce memory usage per connection
            let mut buf = vec![0; 256];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Convert the received bytes to a string and print it
                let received_msg = String::from_utf8_lossy(&buf[0..n]);
                println!("Client {:?} sent: {}", addr, received_msg);

                // Append "00000000000" to the message
                let response_msg = format!("{}00000000000", received_msg);

                // Write the modified data back to the client
                if let Err(e) = socket.write_all(response_msg.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
            // Permit is automatically released when _permit goes out of scope
        });
    }
}