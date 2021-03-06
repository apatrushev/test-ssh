use std::sync::Arc;

use anyhow::{bail, Context, Result};
use thrussh::client::{self, Handle};
use thrussh_keys::key;
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt, BufWriter},
    join,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use log::info;

#[allow(dead_code)]
pub struct Client {}
impl client::Handler for Client {
    type Error = thrussh::Error;
    type FutureUnit = futures::future::Ready<Result<(Self, client::Session), Self::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, bool), Self::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        futures::future::ready(Ok((self, b)))
    }
    fn finished(self, session: client::Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, session)))
    }
    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        info!("check_server_key: {:?}", server_public_key);
        self.finished_bool(true)
    }
}

pub async fn create_socks5_server(address: &str, username: &str, password: &str) -> Result<bool> {
    let config = thrussh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};

    let mut session = thrussh::client::connect(config, address, sh).await?;
    let _ = session
        .authenticate_password(username, password)
        .await?;

    let session = Arc::new(Mutex::new(session));

    let resolver =
        TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default())?;
    let resolver = Arc::new(resolver);
    let listener = TcpListener::bind("127.0.0.1:3894").await?;
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let resolver = resolver.clone();
        let session = session.clone();
        tokio::spawn(async move {
            let _ = handle_socks5_server_connection(stream, &resolver, &session)
                .await
                .map_err(|e| info!("err: {:?}", e));
        });
    }
}

pub async fn handle_socks5_server_connection(
    stream: TcpStream,
    resolver: &TokioAsyncResolver,
    ssh_session: &Arc<Mutex<Handle<Client>>>,
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
            info!("ipv4 requested");
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
            let i = i
                .iter()
                .next()
                .context("unable to get ip for domain name")?;

            let ip = i.to_string();

            wh.write(&[0x05, 0x00, 0x00, 0x03]).await?;
            wh.write(&buf[0..len]).await?;
            wh.flush().await?;

            info!("connecting to {}", &i);
            let mut chan = ssh_session
                .lock()
                .await
                .channel_open_direct_tcpip(i.to_string(), bb as u32, "127.0.0.1", 80)
                .await?;

            let mut buf = [0u8; 16384];
            loop {
                tokio::select! {
                    s = rh.read(&mut buf) => {
                        if s.is_ok() {
                            let size = s.unwrap();
                            info!("received data from client {:?}", &buf[0..size]);
                            if size == 0 {
                                break
                            };
                            chan.data(&buf[0..size]).await.unwrap();
                        } else {
                            info!("error occured: {:?}", s.unwrap_err());
                            chan.cancel_tcpip_forward(false, ip, bb as u32)
                                .await
                                .unwrap();
                            break;
                        }
                    },
                    Some(msg) = chan.wait() => {
                        match msg {
                            thrussh::ChannelMsg::Data { ref data } => {
                                info!("received data {:?}", &data);
                                wh.write(data).await.unwrap();
                                wh.flush().await.unwrap();
                            }
                            thrussh::ChannelMsg::ExitSignal {
                                signal_name: _,
                                core_dumped: _,
                                error_message: _,
                                lang_tag: _,
                            } => {
                                wh.shutdown().await.unwrap();
                            }
                            _ => {}
                        }
                    },
                }
            }
        }
        0x04 => {
            info!("ipv6 requested");
        }
        _ => bail!("unsupported address type received"),
    }

    Ok(true)
}

#[tokio::test]
async fn test_socks5_server() {
    create_socks5_server().await.unwrap();
}
