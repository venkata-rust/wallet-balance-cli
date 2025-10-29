//! Arbitrum L2 wallet balance checking functionality
//!
//! This module provides functions to check Arbitrum L2 wallet balances
//! using Arbitrum's public RPC endpoint.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::WalletBalance;

const ARBITRUM_RPC_URL: &str = "https://arb1.arbitrum.io/rpc";

/// JSON-RPC request structure
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<serde_json::Value>,
    id: u64,
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Get Arbitrum L2 wallet balance for a given address
///
/// # Arguments
///
/// * `address` - Ethereum address to check on Arbitrum network
///
/// # Returns
///
/// Returns a `WalletBalance` containing the balance in ETH
pub async fn get_balance(address: &str) -> Result<WalletBalance> {
    let address = normalize_address(address)?;
    validate_address(&address)?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_getBalance".to_string(),
        params: vec![json!(address), json!("latest")],
        id: 1,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(ARBITRUM_RPC_URL)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Arbitrum RPC")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "RPC request failed with status: {}",
            response.status()
        ));
    }

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse JSON response from Arbitrum RPC")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!(
            "RPC error {}: {}",
            error.code,
            error.message
        ));
    }

    let balance_hex = rpc_response
        .result
        .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))?;

    // Convert hex balance (in wei) to ETH
    let balance_wei = parse_hex_to_u128(&balance_hex)?;
    let balance_eth = wei_to_eth(balance_wei);

    Ok(WalletBalance::new(
        address.to_string(),
        balance_eth,
        "arbitrum".to_string(),
        "ETH".to_string(),
    ))
}

/// Normalize Ethereum address by ensuring it has 0x prefix
fn normalize_address(address: &str) -> Result<String> {
    if address.is_empty() {
        return Err(anyhow::anyhow!("Arbitrum address cannot be empty"));
    }

    let normalized = if address.starts_with("0x") || address.starts_with("0X") {
        address.to_lowercase()
    } else {
        format!("0x{}", address.to_lowercase())
    };

    Ok(normalized)
}

/// Validate Ethereum address format (Arbitrum uses same format)
fn validate_address(address: &str) -> Result<()> {
    if !address.starts_with("0x") {
        return Err(anyhow::anyhow!("Arbitrum address must start with 0x"));
    }

    if address.len() != 42 {
        return Err(anyhow::anyhow!(
            "Invalid Arbitrum address length (expected 42 characters)"
        ));
    }

    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow::anyhow!(
            "Arbitrum address contains invalid hex characters"
        ));
    }

    Ok(())
}

/// Parse hex string to u128
fn parse_hex_to_u128(hex_str: &str) -> Result<u128> {
    let hex_str = hex_str.trim_start_matches("0x");
    
    u128::from_str_radix(hex_str, 16)
        .context("Failed to parse hex balance value")
}

/// Convert wei to ETH (1 ETH = 10^18 wei)
fn wei_to_eth(wei: u128) -> String {
    if wei == 0 {
        return "0".to_string();
    }
    
    let eth_whole = wei / 1_000_000_000_000_000_000;
    let eth_fraction = wei % 1_000_000_000_000_000_000;
    
    if eth_fraction == 0 {
        return eth_whole.to_string();
    }
    
    let fraction_str = format!("{:018}", eth_fraction);
    let trimmed = fraction_str.trim_end_matches('0');
    
    format!("{}.{}", eth_whole, trimmed)
}
