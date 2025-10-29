//! Bitcoin wallet balance checking functionality
//!
//! This module provides functions to check Bitcoin wallet balances
//! using the Blockchain.com API.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::WalletBalance;

const BLOCKCHAIN_INFO_API: &str = "https://blockchain.info";

/// Response structure from Blockchain.com API
#[derive(Debug, Deserialize)]
struct BlockchainInfoResponse {
    #[serde(rename = "final_balance")]
    final_balance: u64,
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

    let url = format!("{}/rawaddr/{}", BLOCKCHAIN_INFO_API, address);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "wallet-balance-cli/0.1.0")
        .send()
        .await
        .context("Failed to send request to Blockchain.com API")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let data: BlockchainInfoResponse = response
        .json()
        .await
        .context("Failed to parse JSON response from Blockchain.com API")?;

    // Convert satoshis to BTC (1 BTC = 100,000,000 satoshis)
    let balance_btc = data.final_balance as f64 / 100_000_000.0;

    Ok(WalletBalance::new(
        address.to_string(),
        format!("{:.8}", balance_btc),
        "bitcoin".to_string(),
        "BTC".to_string(),
    ))
}

/// Validate Bitcoin address format (basic validation)
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
