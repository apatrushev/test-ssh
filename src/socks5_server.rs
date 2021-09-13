use std::sync::Arc;

use anyhow::{bail, Context, Result};
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    AsyncResolver, Resolver, TokioAsyncResolver,
};

#[allow(dead_code)]

pub async fn create_socks5_server() -> Result<bool> {
    let resolver =
        TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default())?;
    let resolver = Arc::new(resolver);
    let listener = TcpListener::bind("127.0.0.1:3894").await?;
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let resolver = resolver.clone();
        tokio::spawn(async move {
            let _ = handle_socks5_server_connection(stream, &resolver)
                .await
                .map_err(|e| println!("err: {:?}", e));
        });
    }
}

pub async fn handle_socks5_server_connection(
    stream: TcpStream,
    resolver: &TokioAsyncResolver,
) -> Result<bool, anyhow::Error> {
    let (mut rh, wh) = split(stream);
    let mut wh = BufWriter::new(wh);

    let mut buf = [0u8; 4];
    let _len = rh.read(&mut buf).await?;
    wh.write_all(&[0x05, 0x00]).await?;
    wh.flush().await?;
    let mut buf = [0u8; 4];
    let _len = rh.read_exact(&mut buf).await?;

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
            let mut buf = [0u8; 128];
            let len = rh.read(&mut buf).await?;
            let domain_name = &buf[1..len - 2];
            let port = &buf[len - 2..];
            let bb = ((port[0] as u16) << 8) | (port[1] as u16);

            let i = resolver
                .ipv4_lookup(String::from_utf8_lossy(domain_name).as_ref())
                .await
                .with_context(|| {
                    format!("unable to resolve {}", String::from_utf8_lossy(domain_name))
                })?;
            let i = i.iter().next().context("unable")?;

            let addr = format!("{}:{}", i, &bb);
            wh.write(&[0x05, 0x00, 0x00, 0x03]).await?;
            wh.write(&buf[0..len]).await?;
            wh.flush().await?;

            let mut endpoint = TcpStream::connect(&addr)
                .await
                .context("failed to connect to endpoint")?;
            let mut stream = rh.unsplit(wh.into_inner());
            let r = tokio::io::copy_bidirectional(&mut stream, &mut endpoint).await;
            match r {
                Ok(d) => {
                    println!("done {}:{}", d.0, d.1);
                }
                Err(e) => {
                    endpoint.shutdown().await?;
                    stream.shutdown().await?;
                    bail!("error occured: {:#?}", &e);
                }
            }
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
