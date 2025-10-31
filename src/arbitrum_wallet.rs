//! Arbitrum L2 wallet balance checking functionality
//!
//! This module provides functions to check Arbitrum L2 wallet balances
//! using Arbitrum's public RPC endpoint.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use hex::encode as hex_encode;

use crate::WalletBalance;

const ARBITRUM_RPC_URL: &str = "https://arb1.arbitrum.io/rpc";


// ERC20 balanceOf function selector: first 4 bytes of keccak256("balanceOf(address)")
const BALANCE_OF_SELECTOR: &str = "70a08231";

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

/// Get ERC20 token balance of a wallet on Arbitrum
///
/// # Arguments
///
/// * `token_address` - ERC20 token contract address (0x prefixed)
/// * `wallet_address` - Wallet address to check balance for (0x prefixed)
///
/// # Returns
///
/// Returns token balance as a decimal string (assumes token has 18 decimals)
pub async fn get_erc20_balance(token_address: &str, wallet_address: &str) -> Result<String> {
    let token_address = normalize_address(token_address)?;
    let wallet_address = normalize_address(wallet_address)?;
    validate_address(&token_address)?;
    validate_address(&wallet_address)?;

    // ABI encode call data: selector + padded wallet address (20 bytes)
    // balanceOf(address) signature: 0x70a08231 + 12 bytes zero + 20 bytes wallet address

    let mut data = hex::decode(BALANCE_OF_SELECTOR).unwrap();
    let wallet_clean = wallet_address.trim_start_matches("0x");

    // Pad the wallet address to 32 bytes (left padded with zeros)
    let padded_wallet = hex::decode(format!("{:0>64}", wallet_clean))
        .context("Invalid wallet address for ABI encoding")?;
    data.extend(padded_wallet);

    let call_data = format!("0x{}", hex_encode(&data));

    // Prepare eth_call JSON-RPC request
    #[derive(Serialize)]
    struct EthCallParams {
        to: String,
        data: String,
    }

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_call".to_string(),
        params: vec![json!(EthCallParams { to: token_address, data: call_data }), json!("latest")],
        id: 1,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(ARBITRUM_RPC_URL)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send eth_call request")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("eth_call failed with status: {}", response.status()));
    }

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse JSON response")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!("eth_call RPC error {}: {}", error.code, error.message));
    }

    let balance_hex = rpc_response
        .result
        .ok_or_else(|| anyhow::anyhow!("No result in eth_call response"))?;

    let balance_wei = parse_hex_to_u128(&balance_hex)?;
    // For ERC20 tokens, decimals may vary, but assuming 18 decimals by default here
    let balance_decimal = wei_to_eth(balance_wei);

    Ok(balance_decimal)
}
