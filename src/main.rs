use tokio::net::{TcpListener, TcpStream};

use stream::{StreamHandler, Value};
mod stream;
mod storage;

#[tokio::main]
async fn main() {
    let port = std::env::args().nth(2).unwrap_or("6379".to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await.unwrap();

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
    let mut stg = storage::Storage::new();

    loop {
        let command = handler.read_request().await.unwrap();

        let response = if let Some(command) = command {
            let (cmd, args) = extract_command(command).unwrap();
            match cmd.as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                "SET" => {
                    let key = unpack_bulk_str(args[0].clone()).unwrap();
                    let value = unpack_bulk_str(args[1].clone()).unwrap();

                    if args.len() > 3 {
                        let subcommand = unpack_bulk_str(args[2].clone()).unwrap();
                        match subcommand.as_str() {
                            "px" => {
                                let expires = unpack_bulk_str(args[3].clone()).unwrap().parse().unwrap();
                                stg.set(&key, &value, expires);
                            }
                            _ => panic!("Cannot handle subcommand {}", subcommand),
                        }
                    }
                    else {
                        stg.set(&key, &value, 0);
                    }
                    Value::SimpleString("OK".to_string())
                }
                "GET" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    match stg.get(&key) {
                        Some(item) => Value::BulkString(item.value.clone()),
                        None => Value::Null,
                    }
                }
                c => panic!("Cannot handle command {}", c),
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


