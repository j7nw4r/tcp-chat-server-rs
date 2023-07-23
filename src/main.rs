use anyhow::{bail, Context};
use std::net::{IpAddr, Ipv4Addr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::broadcast::{error::RecvError, Receiver, Sender},
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let tcp_listerner = tokio::net::TcpListener::bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), 23234))
        .await
        .context("could not create TcpListener")?;
    let (sender, _) = tokio::sync::broadcast::channel::<String>(2);

    loop {
        let (tcp_stream, addr) = tcp_listerner
            .accept()
            .await
            .context("could not accept from tcp_listener")?;
        info!("received connection from  {}", addr);

        let sender = sender.clone();
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

async fn process_stream(stream: TcpStream, sender: Sender<String>) -> anyhow::Result<()> {
    let (tcp_reader, tcp_writer) = stream.into_split();
    let (broadcast_sender, broadcast_receiver) = (sender.clone(), sender.subscribe());
    match tokio::select! {
        read_tcp_write_chat_result = read_tcp_write_chat(tcp_reader, broadcast_sender) => {
            info!("read_tcp_write_chat_result returned");
            read_tcp_write_chat_result
        },
        read_chat_write_tcp_result = read_chat_write_tcp( tcp_writer, broadcast_receiver) => {
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

async fn read_tcp_write_chat(
    mut tcp_reader: OwnedReadHalf,
    chat_sender: Sender<String>,
) -> anyhow::Result<()> {
    loop {
        let mut buf: Vec<u8> = vec![0u8; 1024];
        tcp_reader
            .readable()
            .await
            .context("could not determine if tcp stream was readable")?;

        let _read_length = tcp_reader.read(&mut buf).await?;
        let msg = String::from_utf8(buf)?;
        if let Err(e) = chat_sender.send(msg) {
            error!("send_result : {}", e);
            bail!(e);
        }
    }
}
async fn read_chat_write_tcp(
    mut tcp_writer: OwnedWriteHalf,
    mut chat_receiver: Receiver<String>,
) -> anyhow::Result<()> {
    loop {
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
        tcp_writer
            .write_all(format!("received: {}", msg).as_bytes())
            .await?;
    }
}
