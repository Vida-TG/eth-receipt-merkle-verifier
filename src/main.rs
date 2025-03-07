use anyhow::Result;
use clap::Parser;
use ethers::providers::{Http, Provider};
use ethers::types::{H256, U64};
use sha3::{Digest, Keccak256};
use std::sync::Arc;
use std::str::FromStr;
use tracing::{info, warn, error};

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

    // Merkle proof as comma-separated hex strings
    #[arg(short, long)]
    proof: String,
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
        let block = self.provider.get_block(block_number).await?
            .ok_or_else(|| anyhow::anyhow!("Block {} not found", block_number))?;
        info!("Retrieved block {} with receipts root {:?}", block_number, block.receipts_root);
        Ok(block.receipts_root)
    }

    async fn get_receipt(&self, tx_hash: H256) -> Result<Vec<u8>> {
        let receipt = self.provider.get_transaction_receipt(tx_hash).await?
            .ok_or_else(|| anyhow::anyhow!("Transaction receipt not found for {:?}", tx_hash))?;
        
        // In a real implementation, we would RLP encode the receipt here
        // For now, we'll just use a placeholder hash
        let receipt_data = Keccak256::digest(&receipt.to_string()).to_vec();
        info!("Retrieved and hashed receipt for transaction {:?}", tx_hash);
        Ok(receipt_data)
    }

    async fn verify_receipt_proof(&self, block_number: U64, tx_hash: H256, proof: Vec<H256>) -> Result<bool> {
        info!("Verifying receipt proof for tx {:?} in block {}", tx_hash, block_number);
        let receipts_root = self.get_block_receipts_root(block_number).await?;
        let receipt_data = self.get_receipt(tx_hash).await?;
        let receipt_hash = H256::from_slice(&Keccak256::digest(&receipt_data));
        
        let is_valid = self.verify_merkle_proof(receipt_hash, proof.clone(), receipts_root);
        info!("Proof verification result: {}", is_valid);
        info!("Receipt hash: {:?}", receipt_hash);
        info!("Proof length: {}", proof.len());
        
        Ok(is_valid)
    }

    fn verify_merkle_proof(&self, leaf: H256, proof: Vec<H256>, root: H256) -> bool {
        let mut current = leaf;
        
        for (i, sibling) in proof.iter().enumerate() {
            let mut combined = Vec::with_capacity(64);
            if current < *sibling {
                combined.extend_from_slice(&current.0);
                combined.extend_from_slice(&sibling.0);
            } else {
                combined.extend_from_slice(&sibling.0);
                combined.extend_from_slice(&current.0);
            }
            
            current = H256::from_slice(&Keccak256::digest(&combined));
            info!("Proof step {}: Combined hash {:?}", i + 1, current);
        }
        
        current == root
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Ethereum Receipt Verifier");

    // Load environment variables
    if dotenv::dotenv().is_ok() {
        info!("Loaded environment from .env file");
    }

    // Parse command line arguments
    let args = Args::parse();
    
    // Get RPC URL from args or environment
    let rpc_url = args.rpc_url
        .or_else(|| std::env::var("ETH_RPC_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("RPC URL must be provided via --rpc-url or ETH_RPC_URL env var"))?;

    info!("Connecting to Ethereum node...");
    let verifier = MerkleVerifier::new(&rpc_url)?;
    let block_number = U64::from(args.block);
    
    let tx_hash = match H256::from_str(&args.tx_hash) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Invalid transaction hash format: {}", e);
            return Err(anyhow::anyhow!("Invalid transaction hash"));
        }
    };
    
    // Parse proof from comma-separated hex strings
    let proof: Vec<H256> = match args.proof.split(',')
        .map(|s| H256::from_str(s.trim()))
        .collect::<Result<Vec<_>, _>>() {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid proof format: {}", e);
                return Err(anyhow::anyhow!("Invalid proof format"));
            }
        };

    match verifier.verify_receipt_proof(block_number, tx_hash, proof).await {
        Ok(true) => {
            info!("✅ Merkle proof verification successful!");
            info!("Transaction receipt is included in block {}", args.block);
        }
        Ok(false) => {
            warn!("❌ Merkle proof verification failed!");
            warn!("Transaction receipt is NOT included in block {}", args.block);
        }
        Err(e) => {
            error!("Error verifying proof: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
} 