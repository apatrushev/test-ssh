use std::io::{Read, Write};

mod socks5_server;

use anyhow::Result;
use ssh2::Session;
use tokio::{io::split, join, net::TcpStream};

use crate::socks5_server::create_socks5_server;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    create_socks5_server().await.unwrap();
}

#[tokio::test]
async fn test_connection() {
    let e = test_ssh_connection().await;
    if e.is_err() {
        println!("{:#}", e.unwrap_err());
    }
}

async fn test_ssh_connection() -> Result<bool> {
    let tcp = TcpStream::connect("40.91.208.240:22").await?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    sess.userauth_password("work", "AonxIZonx8d7381oQNDO")?;
    sess.authenticated();
    let c = sess.channel_direct_tcpip("3.233.172.144", 80, None)?;

    // here we'll try to create two tokio threads and implement requests to different urls
    let h = tokio::spawn(async move {
        let mut s = c.stream(0);
        let data = "GET /ip HTTP/1.1\r\n
    Host: httpbin.org\r\n
    User-Agent: curl/7.64.1\r\n
    Accept: */*\r\n\r\n";
        s.write_all(data.as_bytes()).unwrap();

        let mut buf = vec![];
        let w = s.read_to_end(&mut buf).unwrap();
        println!("read {} {:?}", w, &buf[0..w]);
        println!(
            "data:
        {:?}",
            String::from_utf8_lossy(&buf)
        );
    });

    join!(h).0?;

    Ok(true)
}
