extern crate chat_base;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use chat_base::{MessageType, User, PORT};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_socket = TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)).await?;
    let users = Arc::new(Mutex::new(HashMap::<SocketAddr, User>::new()));
    loop {
        let (client_socket, client_addr) = server_socket.accept().await?;

        println!(
            "New connection: {}:{}",
            client_addr.ip(),
            client_addr.port()
        );

        let users_copy = Arc::clone(&users);
        tokio::spawn(async move { handle_client(client_socket, client_addr, users_copy).await });
    }
}

async fn handle_client(
    mut client_socket: TcpStream,
    client_addr: SocketAddr,
    users: Arc<Mutex<HashMap<SocketAddr, User>>>,
) {
    println!("Handling {}:{}", client_addr.ip(), client_addr.port());
    let mut buf = [0_u8; 1024];
    loop {
        match client_socket.read(&mut buf).await {
            Err(e) => {
                eprintln!("Failed to read from socket: {e}");
                return;
            }
            _ => {}
        };
        let message = match rmp_serde::from_slice::<MessageType>(&buf) {
            Ok(message) => message,
            Err(err) => {
                eprintln!("Failed to decode message: {err}");
                return;
            }
        };
        match message {
            MessageType::JOIN(user) => {
                println!("User {} has joined", user.name);
                (*users.lock().unwrap()).insert(client_addr, user);
            }
            MessageType::MESSAGE(user, msg) => {
                println!("User {} messages {msg}", user.name);
            }
            MessageType::LEAVE(user) => {
                println!("User {} has left", user.name);
                (*users.lock().unwrap()).retain(|_, v| {
                    return *v != user;
                });
                break;
            }
        }
    }
    let _ = client_socket.shutdown().await;
}
