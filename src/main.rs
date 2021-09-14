mod socks5_server;
use crate::socks5_server::create_socks5_server;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    create_socks5_server().await.unwrap();
}

#[tokio::test]
async fn test_connection() {
    // let e = test_ssh_connection().await;
    // if e.is_err() {
    //     println!("{:#}", e.unwrap_err());
    // }
}
