extern crate chat_base;
use std::
    net::Ipv4Addr
;

use chat_base::{Message, MessageType, ServerInfo, ServerResponse, PORT};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Receiver, Sender},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_socket = TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)).await?;

    let (tx, _) = broadcast::channel::<ServerInfo>(1024);

    // TODO: Implement channels to send the message to client
    // let (mut tx, rx) = ms

    loop {
        let (client_socket, client_addr) = server_socket.accept().await?;

        println!(
            "New connection: {}:{}",
            client_addr.ip(),
            client_addr.port()
        );

        let tx_clone = tx.clone();
        let rx_clone = tx.subscribe();
        tokio::spawn(async move { handle_client(client_socket, tx_clone, rx_clone).await });

        // tokio::select! {
        //
        // }
    }
}

async fn handle_client(
    mut client_socket: TcpStream,
    tx: Sender<ServerInfo>,
    mut rx: Receiver<ServerInfo>,
) {
    let mut buf = [0_u8; 1024];
    loop {
        tokio::select! {
            Ok(buf) = async {
                let _ = client_socket.read(&mut buf).await?;
                Ok::<_, io::Error>(buf)
            } => {
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
                        let ok_msg = match rmp_serde::to_vec(&ServerResponse::Ok) {
                            Ok(message) => message,
                            Err(err) => {
                                eprintln!("Failed to encode message: {err}");
                                return;

                            }
                        };
                        let _ = client_socket.write_all(&ok_msg).await;
                        let _ = tx.send(ServerInfo::Join(user));
                    }
                    MessageType::MESSAGE(user, msg) => {
                        println!("User {} messages {msg}", user.name);
                        let message = Message{user, text:msg};
                        let _ = tx.send(ServerInfo::Messaged(message));
                    }
                    MessageType::LEAVE(user) => {
                        println!("User {} has left", user.name);
                        let _ = tx.send(ServerInfo::Left(user));
                        break;
                    }
                }
            },
            res = rx.recv() => {
                let msg_message = rmp_serde::to_vec(&res.unwrap());
                let _ = client_socket.write_all(&msg_message.unwrap()).await;
            }
        }
    }
    let _ = client_socket.shutdown().await;
}
