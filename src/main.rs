use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to Read from client");
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Recived request: {}", request);
    let response = "Hello Client!".as_bytes();
    stream.write(response).expect("Failed to write response");
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to entry point");
    print!("Coneccted");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprint!("Failed to establish connection: {}", e)
            }
        }
    }
}
