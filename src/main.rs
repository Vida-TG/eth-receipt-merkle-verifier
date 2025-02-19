use anyhow::Result;
use clap::Parser;
use ethers::providers::{Http, Provider};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rpc_url: Option<String>,

    // Block number to verify
    #[arg(short, long)]
    block: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();
    
    let rpc_url = args.rpc_url
        .or_else(|| std::env::var("ETH_RPC_URL").ok())
        .expect("RPC URL must be provided via --rpc-url or ETH_RPC_URL env var");

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let provider = Arc::new(provider);
    
    println!("Connected to Ethereum node!");
    println!("Will verify block: {}", args.block);
    
    Ok(())
} 