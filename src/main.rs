mod receiver;
mod transmitter;
use receiver::receiver;
use std::io::Write;
use std::path::Path;
use tokio::time::{Duration, sleep};
use transmitter::transmitter;
fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("key.pem exists: {}", Path::new("./ssl/key.pem").exists());
    println!("cert.pem exists: {}", Path::new("./ssl/cert.pem").exists());
    println!("working dir: {:?}", std::env::current_dir().unwrap());
    let ip_local = "0.0.0.0";
    let port_local = read_input("Input Current port: ");

    let receiver_addr = format!("{}:{}", ip_local, port_local);

    println!("Receiver address: {}", receiver_addr.clone());
    tokio::spawn(receiver(receiver_addr));

    // Give the server a moment to start
    sleep(Duration::from_millis(100)).await;

    let ip_remote = read_input("Input Current ip: ");
    let port_remote = read_input("Input Current port: ");
    transmitter(ip_remote, port_remote).await;
}
