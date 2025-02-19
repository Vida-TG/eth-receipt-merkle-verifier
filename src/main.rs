use anyhow::Result;
use clap::Parser;
use ethers::providers::{Http, Provider};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Ethereum RPC URL
    #[arg(short, long)]
    rpc_url: String,

    // Block number to verify
    #[arg(short, long)]
    block: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let provider = Provider::<Http>::try_from(args.rpc_url)?;
    let provider = Arc::new(provider);
    
    println!("Connected to Ethereum node!");
    println!("Will verify block: {}", args.block);
    
    Ok(())
} 