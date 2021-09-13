use anyhow::{bail, Context, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub async fn create_socks5_server() -> Result<bool> {
    let listener = TcpListener::bind("127.0.0.1:3894").await?;
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let _ = handle_socks5_server_connection(stream)
                .await
                .map_err(|e| println!("err: {:#?}", e));
        });
    }
}

pub async fn handle_socks5_server_connection(stream: TcpStream) -> Result<bool, anyhow::Error> {
    let mut stream = stream;

    println!("new connection: {:?}", &stream.peer_addr()?);
    let mut buf = [0u8; 4];
    stream.read(&mut buf).await?;
    println!("{:?}", &buf);
    // stream
    //     .read_exact(&mut buf)
    //     .await
    //     .with_context(|| "failed to read from stream")?;
    // stream
    //     .write_all(&[0x05, 0x00])
    //     .await
    //     .with_context(|| "failed to write to stream")?;

    // let mut buf = [0u8; 4];
    // println!("{:?}", &buf);
    // stream
    //     .read(&mut buf)
    //     .await
    //     .context("failed to read")?;

    // if buf[0] != 0x05 {
    //     bail!("invalid version received");
    // } else if buf[1] != 0x01 {
    //     bail!("invalid command received");
    // }

    // let atyp = buf[3];

    match buf[3] {
        0x01 => {
            // ipv4 address
            println!("parsing ipv4 address");
        }
        0x03 => {
            println!("parsing domain name");
            // domain name
        }
        0x04 => {
            // ipv6 address
            println!("parsing ipv6 address");
        }
        _ => bail!("invalid address type received."),
    }

    let mut buf = [0u8; 32];
    stream.read(&mut buf).await?;

    println!("{:?}", &buf);
    Ok(true)
}

#[tokio::test]
async fn test_socks5_server() {
    create_socks5_server().await.unwrap();
}
