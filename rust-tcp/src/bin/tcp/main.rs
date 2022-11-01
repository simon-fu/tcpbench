
use anyhow::Result;
use args::{Args, Commands::{Client, Server}};
use clap::Parser;

mod args;
mod client;
mod server;

#[tokio::main]
async fn main() -> Result<()> { 
    let args = Args::parse(); 
    match args.command {
        Client(args) => client::run(args).await,
        Server(args) => server::run(args).await,
    }
}
