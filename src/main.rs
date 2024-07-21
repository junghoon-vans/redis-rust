use tokio::net::{TcpListener, TcpStream};

use stream::{StreamHandler, Value};
mod stream;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");

                tokio::spawn(async move {
                    handle_conn(stream).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_conn(stream: TcpStream) {
    let mut handler = StreamHandler::new(stream);
    loop {
        let command = handler.read_request().await.unwrap();

        let response = if let Some(command) = command {
            let (cmd, args) = extract_command(command).unwrap();
            match cmd.as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                _ => Value::SimpleString("OK".to_string()),
            }
        } else {
            break;
        };

        println!("response: {:?}", response);
        handler.write_response(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>), anyhow::Error> {
    match value {
        Value::Array(a) => {
            Ok((
                unpack_bulk_str(a.first().unwrap().clone())?,
                a.into_iter().skip(1).collect(),
            ))
        }
        _ => Err(anyhow::anyhow!("Invalid command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String, anyhow::Error> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string")),
    }
}


