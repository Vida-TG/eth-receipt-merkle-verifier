use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Block number to verify
    #[arg(short, long)]
    block: u64,
}

fn main() {
    let args = Args::parse();
    println!("Will verify block: {}", args.block);
} 