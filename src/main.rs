use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

mod handler;
mod protocol;
mod session;

use handler::RequestHandler;
use protocol::parse_request;

fn handle_connection(stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    println!("Client Connected: {}", peer_addr);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;
    let mut handler = RequestHandler::new();

    loop {
        let mut request_text = String::new();

        loop {
            let mut line = String::new();

            match reader.read_line(&mut line) {
                Ok(0) => {
                    // Connection closed
                    println!("Client disconnected: {}", peer_addr);
                    return;
                }
                Ok(_) => {
                    request_text.push_str(&line);

                    if line == "\r\n" || line == "\n" {
                        break;
                    }
                }
                Err(e) => {
                    println!("Read error: {}", e);
                    return;
                }
            }
        }

        println!(">>> Received:\n{}", request_text);

        match parse_request(&request_text) {
            Ok(request) => {
                let response = handler.handle(&request);
                let response_bytes = response.serialize();

                println!("<<< Sending:\n{}", response_bytes);

                if let Err(e) = writer.write_all(response_bytes.as_bytes()) {
                    println!("Write error: {}", e);
                    return;
                }
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8554").expect("Failed to bind to port");
    println!("RTSP Server listening on port 8554...");

    for stream in listener.incoming() {
        match stream {
            Ok(connection) => {
                std::thread::spawn(move || {
                    handle_connection(connection);
                });
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }
}
