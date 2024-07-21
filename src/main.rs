use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut buf = [0; 512];
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) => break,
                        Err(e) => {
                            println!("An error occurred while reading: {}:", e);
                            break;
                        }
                        _ => {},
                    }

                    stream.write(b"+PONG\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
