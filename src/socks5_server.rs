use std::{net::Ipv4Addr, str::FromStr};

use anyhow::{bail, Context, Result};
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};

#[allow(dead_code)]

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
    let (mut rh, wh) = split(stream);
    let mut wh = BufWriter::new(wh);

    let mut buf = [0u8; 4];
    let len = rh.read(&mut buf).await?;
    wh.write_all(&[0x05, 0x00]).await?;
    wh.flush().await?;
    let mut buf = [0u8; 4];
    let len = rh.read_exact(&mut buf).await?;

    if buf[0] != 0x05 {
        bail!("invalid version received");
    } else if buf[1] != 0x01 {
        bail!("unsupported command received");
    }

    match buf[3] {
        0x01 => {
            println!("ipv4 requested");
        }
        0x03 => {
            println!("domain name requested");
            let mut buf = [0u8; 32];
            let len = rh.read(&mut buf).await?;
            let domain_name = &buf[1..len - 2];
            let port = &buf[len - 2..];
            let bb = ((port[0] as u16) << 8) | (port[1] as u16);
            let addr = format!("{}:{}", String::from_utf8_lossy(domain_name), &bb);
            wh.write(&[0x05, 0x00, 0x00, 0x03]).await?;
            wh.write(&buf[0..len]).await?;
            wh.flush().await?;

            let mut endpoint = TcpStream::connect(&addr)
                .await
                .context("failed to connect to endpoint")?;
            let mut stream = rh.unsplit(wh.into_inner());
            tokio::io::copy_bidirectional(&mut endpoint, &mut stream)
                .await
                .context("failed to copy bidirectional")?;
        }
        0x04 => {
            println!("ipv6 requested");
        }
        _ => bail!("unsupported address type received"),
    }

    Ok(true)
}

#[tokio::test]
async fn test_socks5_server() {
    create_socks5_server().await.unwrap();
}
