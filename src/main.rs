use anyhow::Result;
use clap::Parser;
use ethers::providers::{Http, Provider};
use ethers::types::{H256, U64};
use sha3::{Digest, Keccak256};
use std::sync::Arc;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rpc_url: Option<String>,

    // Block number to verify
    #[arg(short, long)]
    block: u64,

    // Transaction hash to verify
    #[arg(short, long)]
    tx_hash: String,
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

    async fn get_block_receipts_root(&self, block_number: U64) -> Result<H256> {
        let block = self.provider.get_block(block_number).await?.unwrap();
        Ok(block.receipts_root)
    }

    async fn get_receipt(&self, tx_hash: H256) -> Result<Vec<u8>> {
        let receipt = self.provider.get_transaction_receipt(tx_hash).await?.unwrap();
        
        // In a real implementation, we would RLP encode the receipt here
        // For now, we'll just use a placeholder hash
        Ok(Keccak256::digest(&receipt.to_string()).to_vec())
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
    let block_number = U64::from(args.block);
    let tx_hash = H256::from_str(&args.tx_hash)?;
    
    let receipts_root = verifier.get_block_receipts_root(block_number).await?;
    let receipt_data = verifier.get_receipt(tx_hash).await?;
    
    println!("Connected to Ethereum node!");
    println!("Block {} receipts root: {:?}", args.block, receipts_root);
    println!("Transaction receipt hash: {:?}", H256::from_slice(&receipt_data));
    
    Ok(())
} 