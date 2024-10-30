extern crate chat_base;
use std::io::Write;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread;

use chat_base::{MessageType, ServerInfo, ServerResponse, User, PORT};
use crossterm::{cursor, terminal, ExecutableCommand};
use tokio::io::{self, AsyncReadExt};
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
    let mut stdout = std::io::stdout();
    let mut stdin = BufReader::new(io::stdin());

    let socket_mutex = Arc::new(Mutex::new(
        TcpStream::connect((Ipv4Addr::LOCALHOST, PORT))
            .await
            .expect("Could not find server!"),
    ));

    stdout.write_all("Enter username: ".as_bytes())?;
    stdout.flush()?;

    let mut username = String::new();
    let _ = stdin.read_line(&mut username).await?;
    username = username.trim().to_string();
    let user = User { name: username };

    let join_message =
        rmp_serde::to_vec(&MessageType::JOIN(user.clone())).expect("Couldn't encode join message");

    {
        let mut socket = socket_mutex.lock().await;
        socket.write_all(&join_message).await?;

        let mut buf = [0_u8; 1024];
        let _ = socket.read(&mut buf).await?;
        let message = rmp_serde::from_slice::<ServerResponse>(&buf)?;
        if let ServerResponse::Ok = message {
            println!("Successfully joined!");
        } else if let ServerResponse::Err = message {
            eprintln!("Unsuccessful in joining");
        }
    }

    let (tx, mut rx) = mpsc::channel::<String>(1024);

    thread::spawn(move || {
        normal(tx);
    });
    loop {
        tokio::select! {
            _error = tokio::signal::ctrl_c() => {
                {
                    let mut socket = socket_mutex.lock().await;
                    let leave_message = rmp_serde::to_vec(&MessageType::LEAVE(user.clone()));
                    let _ = socket
                        .write_all(&leave_message.unwrap())
                        .await?;
                }
                break;
            },
            res = rx.recv() => {
                {
                    let mut socket = socket_mutex.lock().await;
                    let msg_message = rmp_serde::to_vec(&MessageType::MESSAGE(user.clone(), res.unwrap()));
                    let _ = socket.write_all(&msg_message.unwrap()).await?;
                }
            },
            Ok(buf) = async {
                let mut socket = socket_mutex.lock().await;
                let mut buf = [0_u8; 1024];
                let _ = socket.read(&mut buf).await?;
                Ok::<_, io::Error>(buf)
            } => {
                {
                    let msg_message = rmp_serde::from_slice::<ServerInfo>(&buf)?;
                    match msg_message {
                        ServerInfo::Messaged(message) => {
                            let _  = stdout.execute(cursor::MoveToPreviousLine(1));
                            println!("\n{} - {}", message.user.name, message.text);
                        },
                        ServerInfo::Left(user) => {
                            let _  = stdout.execute(cursor::MoveToPreviousLine(1));
                            println!("\n{} has left", user.name);
                        },
                        ServerInfo::Join(user) => {
                            let _  = stdout.execute(cursor::MoveToPreviousLine(1));
                            println!("\n{} has joined", user.name);
                        }
                    }
                }
            }
        }
    }

    println!("Quitting...");
    Ok(())
}

fn normal(tx: Sender<String>) {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    loop {
        let mut message = String::new();
        let _ = stdin.read_line(&mut message);
        let _ = stdout.execute(cursor::MoveUp(1));
        let _ = stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine));
        message = message.trim().to_string();
        let _ = tx.blocking_send(message);
    }
}
