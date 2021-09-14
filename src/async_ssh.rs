use std::io::Read;

use anyhow::{Context, Result};
use ssh2::{Channel, Session};
use tokio::net::TcpStream;

pub(crate) struct AsyncSshConnection {
    inner_s: Session,
}

impl AsyncSshConnection {
    pub(crate) async fn create_connection(
        &mut self,
        target_ip: &str,
        target_port: u16,
    ) -> Result<Channel, anyhow::Error> {
        let channel = self
            .inner_s
            .channel_direct_tcpip(target_ip, target_port, None)?;

        println!("channel created!");
        Ok(channel)
    }
}

pub(crate) async fn create_ssh_connection(
    conn_str: &str,
    username: &str,
    password: &str,
) -> Result<AsyncSshConnection, anyhow::Error> {
    let connection = TcpStream::connect(conn_str)
        .await
        .context("ssh: tcp conn failed")?;
    let mut sess = Session::new().context("ssh: new session failed")?;
    sess.set_tcp_stream(connection);
    sess.handshake().context("ssh: handshake failed")?;
    sess.userauth_password(username, password)
        .context("ssh: auth failed")?;

    Ok(AsyncSshConnection { inner_s: sess })
}

#[tokio::test]
async fn test_ssh_create() {
    let mut z = create_ssh_connection("40.91.208.240:22", "work", "wug2DwxqfHR4APZIBI")
        .await
        .unwrap();

    println!("connected!");

    let h = tokio::task::spawn(async move {
        let mut chan = z.create_connection("1.1.1.1", 53).await.unwrap();
        chan.stream(0);
    });

    let h2 = tokio::task::spawn(async move {
        let mut chan = z.create_connection("1.1.1.1", 53).await.unwrap();
        chan.stream(1);
    });
}
