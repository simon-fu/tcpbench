

use clap::{Parser, Subcommand};


#[derive(Parser, Debug, Clone)]
#[clap(name = "tcp", author, about = "tcp bench", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone)]
#[derive(Subcommand)]
pub enum Commands {
    Client(ClientArgs),
    Server(ServerArgs),
}


#[derive(Parser, Debug, Clone)]
#[clap(name = "tcp_client", author, about = "tcp client", long_about = None)]
pub struct ClientArgs {
    #[clap(short = 'c', long = "target", long_help = "target server address to connect, in the format of ip:port")]
    pub target: String,

    #[clap(long = "conn", long_help = "all connections to setup", default_value = "1")]
    pub conns: u32,

    #[clap(long = "cps", long_help = "setup connections rate, connections/second", default_value = "1000")]
    pub cps: u32,

}

#[derive(Parser, Debug, Clone)]
#[clap(name = "tcp_server", author, about = "tcp server", long_about = None)]
pub struct ServerArgs {
    #[clap(long = "bind", long_help = "bind listen address", default_value = "0.0.0.0:11111")]
    pub addr: String,

    #[clap(long = "pps", long_help = "sending packets rate, packets/second", default_value = "5")]
    pub pps: u64,

    #[clap(short = 't', long = "time", long_help = "send duration in seconds ", default_value = "10")]
    pub secs: u64,

    #[clap(short = 'l', long = "len", long_help = "packet length", default_value = "1000")]
    pub packet_len: usize,
}
