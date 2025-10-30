use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use base58::{FromBase58, ToBase58}; // For Base58Check
use sha2::{Digest, Sha256};

use crate::WalletBalance;

const TRON_API_URL: &str = "https://api.trongrid.io"; // Switch to "https://api.shasta.trongrid.io" for testnet (no key needed)

#[derive(Debug, Deserialize)]
struct AccountResponse {
    success: bool,
    data: Vec<AccountData>,
}

#[derive(Debug, Deserialize)]
struct AccountData {
    balance: Option<u64>,
}

pub async fn get_balance(address: &str) -> Result<WalletBalance> {
    let address = address.trim();
    validate_address(address)?;

    let url = format!("{}/v1/accounts/{}", TRON_API_URL, address);

    let client = reqwest::Client::new();
    let request = client.get(&url);

    let response = request.send().await?;
    
    // Log the full response for debugging
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("API Error - Status: {}, Body: {}", status, body); // Or use tracing/log crate
        return Err(anyhow::anyhow!(
            "TronGrid API failed: {} - {}",
            status, body
        ));
    }

    let data: AccountResponse = response.json().await.context("Failed to parse JSON")?;

    if !data.success || data.data.is_empty() {
        let balance_sun = 0u64;
        let balance_trx = 0.0;
        // Return zero balance for non-existent accounts (common for new/unfunded wallets)
        Ok(WalletBalance::new(
            address.to_string(),
            format!("{:.6}", balance_trx),
            "tron".to_string(),
            "TRX".to_string(),
        ))
    } else {
        let balance_sun = data.data[0].balance.unwrap_or(0);
        let balance_trx = (balance_sun as f64) / 1_000_000.0;

        Ok(WalletBalance::new(
            address.to_string(),
            format!("{:.6}", balance_trx),
            "tron".to_string(),
            "TRX".to_string(),
        ))
    }
}

fn validate_address(address: &str) -> Result<()> {
    if address.len() != 34 || !address.starts_with('T') {
        return Err(anyhow::anyhow!("Invalid Tron address: must be 34 chars starting with 'T'"));
    }

    // Full Base58Check validation
    // let decoded = address.from_base58().context("Invalid Base58 encoding")?;
    let decoded = address.from_base58()
    .map_err(|_| anyhow::anyhow!("Invalid Base58 encoding"))?;
    if decoded.len() != 25 {
        return Err(anyhow::anyhow!("Invalid decoded length"));
    }
    if decoded[0] != 0x41 {
        return Err(anyhow::anyhow!("Invalid Tron version byte"));
    }

    let payload = &decoded[0..21];
    let provided_checksum = &decoded[21..];

    // Double SHA256 checksum
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let hash1 = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    let expected_checksum = &hasher.finalize()[..4];

    if provided_checksum != expected_checksum {
        return Err(anyhow::anyhow!("Invalid address checksum"));
    }

    Ok(())
}