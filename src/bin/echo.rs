// rustc version 1.77.2
// tokio version 1.38.0, features: full

use dirs::*;
use std::env;
use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

/// This macro sets up a default Tokio runtime, and will execute the main method
/// within it.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir().unwrap();
    let cwd_to_string_lossy: String = String::from(cwd.to_string_lossy());
    println!("{}", cwd_to_string_lossy);
    let local_data_dir = data_local_dir();
    println!("{}", local_data_dir.expect("REASON").display());
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    // loop for accepting incoming connections
    loop {
        let (mut sock, _) = listener.accept().await?;

        // Spawn a async task for each connection.
        // The task handles both read and write operations on the connection.
        tokio::spawn(async move {
            let mut buf = vec![0; 512];

            loop {
                let bytes_read = sock.read(&mut buf).await.expect("failed to read request");

                if bytes_read == 0 {
                    return;
                }

                buf.push(b'\n');

                sock.write_all(&buf[0..bytes_read + 1])
                    .await
                    .expect("failed to write response");

                sock.flush().await.expect("failed to flush response")
            }
        });
    }
}
