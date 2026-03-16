use futures_util::{SinkExt, StreamExt};
use openssl::ssl::{SslAcceptor, SslConnector, SslFiletype, SslMethod};
use openssl::x509::X509;
use std::io::Write;
use std::path::Path;
use std::pin::Pin;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};
use tokio_openssl::SslStream;
use tokio_tungstenite::{accept_async, client_async, tungstenite::Message};
fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

async fn receiver(receiver_addr: String) {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    acceptor
        .set_private_key_file("./ssl/key.pem", SslFiletype::PEM)
        .unwrap();

    acceptor
        .set_certificate_chain_file("./ssl/cert.pem") // or set_certificate_file() can use either
        .unwrap();

    let acceptor = acceptor.build();
    let listener = TcpListener::bind(receiver_addr.as_str())
        .await
        .expect("Failed to bind to entry point");

    println!(
        "Recipient listening on wss://{}/web_socket",
        receiver_addr.as_str()
    );

    while let Ok((stream, _)) = listener.accept().await {
        let sender_ip = stream.peer_addr().unwrap();
        let ssl_ctx = openssl::ssl::Ssl::new(acceptor.context()).unwrap();
        let mut ssl_stream = SslStream::new(ssl_ctx, stream).unwrap();

        Pin::new(&mut ssl_stream).accept().await.unwrap();

        tokio::spawn(async move {
            let ws_stream = accept_async(ssl_stream)
                .await
                .expect("WebSocket handshake failed");
            let (mut write, mut read) = ws_stream.split();

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received from {}: {}", sender_ip, text);
                        write.send(Message::Text(text)).await.unwrap();
                    }
                    Ok(Message::Binary(bin)) => {
                        println!("Received: {:?}", bin);
                        write.send(Message::Binary(bin)).await.unwrap();
                    }
                    Ok(Message::Close(_)) => {
                        println!("Client disconnected");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        println!("Received a ping");
                        write.send(Message::Pong(data)).await.unwrap(); // Ping should reply with Pong
                    }
                    Ok(Message::Pong(_)) => {}
                    Ok(_) => {} //default
                    Err(e) => {
                        println!("Recipient error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
async fn transmitter(ip: String, port: String) {
    let mut connector_builder = SslConnector::builder(SslMethod::tls()).unwrap();

    connector_builder
        .set_private_key_file("./ssl/key.pem", SslFiletype::PEM)
        .unwrap();
    connector_builder
        .set_certificate_chain_file("./ssl/cert.pem")
        .unwrap();

    let ca_cert = X509::from_pem(&std::fs::read("./ssl/cert.pem").unwrap()).unwrap();
    connector_builder
        .cert_store_mut()
        .add_cert(ca_cert)
        .unwrap();

    connector_builder.set_verify(openssl::ssl::SslVerifyMode::PEER);

    let connector = connector_builder.build();
    let stream = TcpStream::connect(format!("{}:{}", ip, port))
        .await
        .expect("TCP connect failed");

    let ssl_config = connector
        .configure()
        .unwrap()
        .verify_hostname(false) // Disable hostname verification if using IP
        .into_ssl(&ip) // Try with IP first
        .unwrap();

    let mut tls_stream = SslStream::new(ssl_config, stream).unwrap();

    // timeout only for out going connections so server can go on
    match tokio::time::timeout(Duration::from_secs(10), Pin::new(&mut tls_stream).connect()).await {
        Ok(Ok(_)) => println!("TLS handshake successful"),
        Ok(Err(e)) => {
            eprintln!("TLS handshake failed: {}", e);
            return;
        }
        Err(_) => {
            eprintln!("TLS handshake timeout");
            return;
        }
    }

    let (socket, _) = client_async(format!("wss://{}:{}/web_socket", ip, port), tls_stream)
        .await
        .expect("WebSocket connect failed");
    println!("Connected to: {}:{}", ip, port);
    println!("Connected to: {}:{}", ip, port);
    let (mut write, mut read) = socket.split();

    let reader = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => println!("sent: {}", msg),
                Err(e) => {
                    println!("sent error: {}", e);
                    break;
                }
            }
        }
    });

    let writer = tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            write.send(Message::Text(line.into())).await.unwrap();
        }
    });

    let _ = tokio::join!(reader, writer);
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
