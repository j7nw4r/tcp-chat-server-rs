use std::{io, time::Duration};

use anyhow::{bail, Context};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::broadcast::{Receiver, Sender},
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let tcp_listerner = tokio::net::TcpListener::bind("127.0.0.1:23234")
        .await
        .context("could not create TcpListener on 127.0.0.1:23234")?;
    let (sender, _) = tokio::sync::broadcast::channel::<String>(5);

    loop {
        let (tcp_stream, addr) = tcp_listerner
            .accept()
            .await
            .context("could not accept from tcp_listener")?;

        info!("received connection from  {}", addr);
        tokio::spawn(process_stream(tcp_stream, sender.clone()));
    }
}
async fn process_stream(stream: TcpStream, sender: Sender<String>) -> anyhow::Result<()> {
    let (tcp_reader, tcp_writer) = stream.into_split();
    let (broadcast_sender, broadcast_receiver) = (sender.clone(), sender.subscribe());
    if let Err(e) = tokio::select! {
        read_result = read_future(tcp_reader, broadcast_sender) => {
            info!("read_result returned");
            read_result
        },
        write_result = write_future( tcp_writer, broadcast_receiver) => {
            info!("write_result returned");
            write_result},
    } {
        error!("{}", e);
    };
    Ok(())
}

async fn read_future(
    mut tcp_reader: OwnedReadHalf,
    broadcast_sender: Sender<String>,
) -> anyhow::Result<()> {
    loop {
        let mut buf = vec![0u8; 1024];
        tcp_reader
            .readable()
            .await
            .context("could not determine if tcp stream was reaable")?;

        let read_length = tcp_reader.read(&mut buf).await?;
        buf.truncate(read_length);
        let msg = String::from_utf8(buf)?;
        info!("read: {}", msg);
        let send_result = broadcast_sender.send(msg);
        if let Err(e) = send_result {
            error!("send_result : {}", e);
        }
    }
}
async fn write_future(
    mut tcp_writer: OwnedWriteHalf,
    mut broadcast_receiver: Receiver<String>,
) -> anyhow::Result<()> {
    loop {
        info!("receiver length is {}", broadcast_receiver.len());
        let msg = broadcast_receiver.recv().await?;
        info!("sending: {}", msg);
        tcp_writer
            .write_all(format!("received: {}", msg).as_bytes())
            .await?;
    }
}
