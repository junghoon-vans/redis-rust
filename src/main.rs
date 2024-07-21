use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((mut stream, _)) => {
                println!("accepted new connection");

                tokio::spawn(async move {
                    let mut buf = [0; 512];
                    loop {
                        match stream.read(&mut buf).await {
                            Ok(0) => break,
                            Err(e) => {
                                println!("An error occurred while reading: {}:", e);
                                break;
                            }
                            _ => {},
                        }

                        stream.write(b"+PONG\r\n").await.unwrap();
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
