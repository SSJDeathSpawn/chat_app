extern crate chat_base;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread;

use chat_base::{MessageType, User, PORT};
use tokio::io::{self};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut socket = match TcpStream::connect((Ipv4Addr::LOCALHOST, PORT)).await {
    //     err @ Err(_) => {
    //         eprintln!("Failed");
    //         return err
    //     },
    //     res => res
    // }?;
    let mut stdout = io::stdout();
    let mut stdin = BufReader::new(io::stdin());

    let socket_mutex = Arc::new(Mutex::new(
        TcpStream::connect((Ipv4Addr::LOCALHOST, PORT))
            .await
            .expect("Could not find server!"),
    ));

    stdout.write_all("Enter username: ".as_bytes()).await?;
    stdout.flush().await?;

    let mut username = String::new();
    let _ = stdin.read_line(&mut username).await?;
    username = username.trim().to_string();
    let user = User { name: username };

    let join_message =
        rmp_serde::to_vec(&MessageType::JOIN(user.clone())).expect("Couldn't encode join message");

    {
        let mut socket = socket_mutex.lock().await;
        socket.write_all(&join_message).await?;
    }
    println!("Joined");

    let (tx, mut rx) = mpsc::channel::<String>(200);

    thread::spawn(move || {
        normal(tx);
    });
    loop {
        tokio::select! {
            _error = tokio::signal::ctrl_c() => {
                {
                    let mut socket = socket_mutex.lock().await;
                    let _ = socket
                        .write_all(&rmp_serde::to_vec(&MessageType::LEAVE(user.clone())).unwrap())
                        .await;
                }
                break;
            },
            res = rx.recv() => {
                println!("Sending message: {}!", res.clone().unwrap());
                {
                    let mut socket = socket_mutex.lock().await;
                    let _ = socket.write_all(&rmp_serde::to_vec(&MessageType::MESSAGE(user.clone(), res.unwrap())).unwrap()).await;
                }
            }
        }
    }

    println!("Quitting...");
    Ok(())
}

fn normal(tx: Sender<String>) {
    loop {
        let stdin = std::io::stdin();
        let mut message = String::new();
        let _ = stdin.read_line(&mut message);
        message = message.trim().to_string();
        let _ = tx.blocking_send(message);
    }
}
