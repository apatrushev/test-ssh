mod socks5_server;
use crate::socks5_server::create_socks5_server;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub(crate) struct Opt {
    /// Listen port
    #[structopt(short, long, default_value = "localhost:22")]
    pub(crate) address: String,

    /// User
    #[structopt(short, long, default_value = "test")]
    pub(crate) username: String,

    /// Password
    #[structopt(short, long, default_value = "")]
    pub(crate) password: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    create_socks5_server(&opt.address, &opt.username, &opt.password).await.unwrap();
}
