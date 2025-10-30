//! Bitcoin wallet balance checking functionality
//!
//! This module provides functions to check Bitcoin wallet balances
//! using the Blockchain.com API.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::WalletBalance;

// const BLOCKCHAIN_INFO_API: &str = "https://blockchain.info";
const BLOCKCHAIN_INFO_API: &str = "https://blockstream.info/api";

//  Response structure from Blockstream.info API
#[derive(Debug, Deserialize)]
struct BlockstreamResponse {
    chain_stats: ChainStats,
}

#[derive(Debug, Deserialize)]
struct ChainStats {
    funded_txo_sum: u64,  // Total received (in satoshis)
    spent_txo_sum: u64,   // Total spent (in satoshis)
}

/// Get Bitcoin wallet balance for a given address
///
/// # Arguments
///
/// * `address` - Bitcoin address to check
///
/// # Returns
///
/// Returns a `WalletBalance` containing the balance in BTC
pub async fn get_balance(address: &str) -> Result<WalletBalance> {
    validate_address(address)?;

    let url = format!("{}/address/{}", BLOCKCHAIN_INFO_API, address);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "wallet-balance-cli/0.1.0")
        .send()
        .await
        .context("Failed to send request to Blockstream API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "API failed: {} - {}",
            status,
            body
        ));
    }

    let data: BlockstreamResponse = response
        .json()
        .await
        .context("Failed to parse JSON from Blockstream")?;

    let balance_sats = data.chain_stats.funded_txo_sum.saturating_sub(data.chain_stats.spent_txo_sum);
    let balance_btc = balance_sats as f64 / 100_000_000.0;

    Ok(WalletBalance::new(
        address.to_string(),
        format!("{:.8}", balance_btc),
        "bitcoin".to_string(),
        "BTC".to_string(),
    ))
}

fn validate_address(address: &str) -> Result<()> {
    if address.is_empty() {
        return Err(anyhow::anyhow!("Bitcoin address cannot be empty"));
    }

    // Basic validation: Bitcoin addresses are typically 26-35 characters
    if address.len() < 26 || address.len() > 62 {
        return Err(anyhow::anyhow!("Invalid Bitcoin address length"));
    }

    // Check if starts with valid prefix (1, 3, or bc1)
    if !address.starts_with('1') 
        && !address.starts_with('3') 
        && !address.starts_with("bc1") {
        return Err(anyhow::anyhow!(
            "Invalid Bitcoin address format (must start with 1, 3, or bc1)"
        ));
    }

    Ok(())
}