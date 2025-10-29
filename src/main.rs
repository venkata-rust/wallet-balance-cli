//! Wallet Balance CLI
//!
//! Command-line tool to check cryptocurrency wallet balances

use clap::Parser;
use std::process;
use wallet_balance::{bitcoin_wallet, ethereum_wallet, base_wallet, Network};

#[derive(Parser)]
#[command(name = "wallet-balance")]
#[command(author = "Your Name")]
#[command(version = "0.1.0")]
#[command(about = "Check cryptocurrency wallet balances", long_about = None)]
struct Cli {
    /// Network to check (bitcoin, ethereum)
    #[arg(short, long, value_name = "NETWORK")]
    network: String,

    /// Wallet address to check
    #[arg(short, long, value_name = "ADDRESS")]
    address: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Parse network
    let network: Network = match cli.network.parse() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Supported networks: bitcoin, ethereum");
            process::exit(1);
        }
    };

    // Fetch balance based on network
    let result = match network {
        Network::Bitcoin => {
            println!("Fetching Bitcoin balance for address: {}", cli.address);
            bitcoin_wallet::get_balance(&cli.address).await
        }
        Network::Ethereum => {
            println!("Fetching Ethereum balance for address: {}", cli.address);
            ethereum_wallet::get_balance(&cli.address).await
        }
         Network::Base => {  // NEW: Add this match arm
            println!("Fetching Base L2 balance for address: {}", cli.address);
            base_wallet::get_balance(&cli.address).await
        }
    };

    // Display result
    match result {
        Ok(balance) => {
            println!("\n✅ Success!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Network:  {}", balance.network.to_uppercase());
            println!("Address:  {}", balance.address);
            println!("Balance:  {} {}", balance.balance, balance.denomination);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        Err(e) => {
            eprintln!("\n❌ Error fetching balance: {}", e);
            eprintln!("\nPlease check:");
            eprintln!("  • Address format is correct");
            eprintln!("  • Network is spelled correctly");
            eprintln!("  • You have internet connectivity");
            process::exit(1);
        }
    }
}
