/// A simple TCP chat server implemented in Rust using Tokio for asynchronous operations.
use anyhow::{bail, Context};
use std::net::{IpAddr, Ipv4Addr};
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::broadcast::{error::RecvError, Receiver, Sender},
};
use tracing::{error, info};

/// The main entry point of the application.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber for logging.
    tracing_subscriber::fmt::init();

    // Bind the TCP listener to the specified IP address and port.
    let tcp_listener = tokio::net::TcpListener::bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), 23234))
        .await
        .context("could not create TcpListener")?;
    
    // Create a broadcast channel for sending messages.
    let (sender, _) = tokio::sync::broadcast::channel::<String>(2);

    loop {
        // Accept incoming TCP connections.
        let (tcp_stream, addr) = tcp_listener
            .accept()
            .await
            .context("could not accept from tcp_listener")?;
        
        info!("received connection from {}", addr);

        // Clone the sender for the new connection.
        let sender = sender.clone();
        
        // Spawn a new task to process the incoming stream.
        tokio::spawn(async move {
            match process_stream(tcp_stream, sender).await {
                Ok(_) => (),
                Err(e) => {
                    error!("process_stream returned error: {}", e)
                }
            }
        });
    }
}

/// Processes the TCP stream by reading from it and writing to the chat.
async fn process_stream(stream: TcpStream, sender: Sender<String>) -> anyhow::Result<()> {
    // Split the TCP stream into a reader and a writer.
    let (tcp_reader, tcp_writer) = stream.into_split();
    let (broadcast_sender, broadcast_receiver) = (sender.clone(), sender.subscribe());
    
    // Use Tokio's select to wait for either reading from TCP or writing to chat.
    match tokio::select! {
        read_tcp_write_chat_result = read_tcp_write_chat(tcp_reader, broadcast_sender) => {
            info!("read_tcp_write_chat_result returned");
            read_tcp_write_chat_result
        },
        read_chat_write_tcp_result = read_chat_write_tcp(tcp_writer, broadcast_receiver) => {
            info!("read_chat_write_tcp_result returned");
            read_chat_write_tcp_result
        },
    } {
        Ok(unit) => Ok(unit),
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}

/// Reads messages from the TCP stream and sends them to the chat.
async fn read_tcp_write_chat(
    tcp_reader: OwnedReadHalf,
    chat_sender: Sender<String>,
) -> anyhow::Result<()> {
    loop {
        let mut buf: Vec<u8> = vec![0u8; 1024];
        
        // Wait until the TCP stream is readable.
        tcp_reader
            .readable()
            .await
            .context("could not determine if tcp stream was readable")?;

        
        // Convert the bytes to a UTF-8 string.
        let msg = String::from_utf8(buf)?;
        
        // Send the message to the chat.
        if let Err(e) = chat_sender.send(msg) {
            error!("send_result : {}", e);
            bail!(e);
        }
    }
}

/// Receives messages from the chat and writes them to the TCP stream.
async fn read_chat_write_tcp(
    mut tcp_writer: OwnedWriteHalf,
    mut chat_receiver: Receiver<String>,
) -> anyhow::Result<()> {
    loop {
        // Wait for a message from the chat.
        let msg = match chat_receiver.recv().await {
            Ok(msg) => msg,
            Err(e) => {
                if let RecvError::Lagged(num_of_skip) = e {
                    info!("lagged. skipped {} messages", num_of_skip);
                    continue;
                }
                bail!(e)
            }
        };
        
        // Write the received message to the TCP stream.
        tcp_writer
            .write_all(format!("received: {}", msg).as_bytes())
            .await?;
    }
}
