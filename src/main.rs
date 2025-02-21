use anyhow::Result;
use clap::Parser;
use ethers::providers::{Http, Provider};
use ethers::types::H256;
use sha3::{Digest, Keccak256};
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

struct MerkleVerifier {
    provider: Arc<Provider<Http>>,
}

impl MerkleVerifier {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    fn verify_merkle_proof(&self, leaf: H256, proof: Vec<H256>, root: H256) -> bool {
        let mut current = leaf;
        
        for sibling in proof {
            let mut combined = Vec::with_capacity(64);
            if current < sibling {
                combined.extend_from_slice(&current.0);
                combined.extend_from_slice(&sibling.0);
            } else {
                combined.extend_from_slice(&sibling.0);
                combined.extend_from_slice(&current.0);
            }
            
            current = H256::from_slice(&Keccak256::digest(&combined));
        }
        
        current == root
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();
    
    let rpc_url = args.rpc_url
        .or_else(|| std::env::var("ETH_RPC_URL").ok())
        .expect("RPC URL must be provided via --rpc-url or ETH_RPC_URL env var");

    let verifier = MerkleVerifier::new(&rpc_url)?;
    println!("Connected to Ethereum node!");
    println!("Will verify block: {}", args.block);
    
    Ok(())
} 