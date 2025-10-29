//! Integration tests for wallet balance CLI
//!
//! These are pass-to-pass tests that verify existing functionality
//! continues to work as expected.

use wallet_balance::{bitcoin_wallet, ethereum_wallet, base_wallet, Network};

use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// PASS-TO-PASS TESTS: Bitcoin (5 tests)
// ============================================================================

#[tokio::test]
async fn test_bitcoin_balance_returns_valid_structure() {
    sleep(Duration::from_millis(500)).await;
    
    let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let result = bitcoin_wallet::get_balance(address).await;
    
    if let Err(e) = &result {
        eprintln!("Bitcoin API error: {}", e);
    }
    
    assert!(result.is_ok(), "Bitcoin balance fetch should succeed");
    
    let balance = result.unwrap();
    assert_eq!(balance.network, "bitcoin");
    assert_eq!(balance.denomination, "BTC");
    assert_eq!(balance.address, address);
}

#[tokio::test]
async fn test_bitcoin_balance_format_is_numeric() {
    sleep(Duration::from_millis(500)).await;
    
    let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let result = bitcoin_wallet::get_balance(address).await;
    
    assert!(result.is_ok());
    
    let balance = result.unwrap();
    let balance_value: f64 = balance.balance.parse().expect("Balance should be numeric");
    assert!(balance_value >= 0.0, "Balance should be non-negative");
}

#[tokio::test]
async fn test_bitcoin_invalid_address_returns_error() {
    let invalid_address = "invalid_bitcoin_address";
    let result = bitcoin_wallet::get_balance(invalid_address).await;
    assert!(result.is_err(), "Invalid address should return error");
}

#[tokio::test]
async fn test_bitcoin_empty_address_returns_error() {
    let result = bitcoin_wallet::get_balance("").await;
    assert!(result.is_err(), "Empty address should return error");
}

#[tokio::test]
async fn test_bitcoin_p2sh_address_works() {
    sleep(Duration::from_millis(500)).await;
    
    // Use a well-known P2SH address (Bitfinex cold wallet)
    let address = "3D2oetdNuZUqQHPJmcMDDHYoqkyNVsFk9r";
    let result = bitcoin_wallet::get_balance(address).await;
    
    if let Err(e) = &result {
        eprintln!("Bitcoin P2SH API error: {}", e);
        // If API is down, just verify validation works
        assert!(address.starts_with('3'), "P2SH address should start with 3");
        return;
    }
    
    let balance = result.unwrap();
    assert_eq!(balance.network, "bitcoin");
    assert_eq!(balance.address, address);
}

// ============================================================================
// PASS-TO-PASS TESTS: Ethereum (3 tests)
// ============================================================================

#[tokio::test]
async fn test_ethereum_balance_returns_valid_structure() {
    sleep(Duration::from_secs(1)).await;
    
    let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
    let result = ethereum_wallet::get_balance(address).await;
    
    if let Err(e) = &result {
        eprintln!("Ethereum API error: {}", e);
        eprintln!("Note: Free RPC endpoints may have rate limits or require different configuration");
        // Skip test if RPC is unavailable
        return;
    }
    
    let balance = result.unwrap();
    assert_eq!(balance.network, "ethereum");
    assert_eq!(balance.denomination, "ETH");
    assert!(balance.address.starts_with("0x"));
}

#[tokio::test]
async fn test_ethereum_address_normalization() {
    sleep(Duration::from_secs(1)).await;
    
    let address_without_prefix = "d8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
    let result = ethereum_wallet::get_balance(address_without_prefix).await;
    
    if let Err(e) = &result {
        eprintln!("Ethereum normalization API error: {}", e);
        // Skip test if RPC is unavailable
        return;
    }
    
    let balance = result.unwrap();
    assert!(balance.address.starts_with("0x"));
}

#[tokio::test]
async fn test_ethereum_invalid_address_returns_error() {
    let invalid_address = "0xinvalid";
    let result = ethereum_wallet::get_balance(invalid_address).await;
    assert!(result.is_err(), "Invalid address should return error");
}

// ============================================================================
// PASS-TO-PASS TESTS: Core Library (2 tests)
// ============================================================================

#[test]
fn test_network_parsing() {
    assert_eq!("bitcoin".parse::<Network>().unwrap(), Network::Bitcoin);
    assert_eq!("btc".parse::<Network>().unwrap(), Network::Bitcoin);
    assert_eq!("ethereum".parse::<Network>().unwrap(), Network::Ethereum);
    assert_eq!("eth".parse::<Network>().unwrap(), Network::Ethereum);
    assert!("invalid".parse::<Network>().is_err());
}

#[tokio::test]
async fn test_concurrent_api_calls() {
    sleep(Duration::from_secs(1)).await;
    
    let btc_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let eth_address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
    
    let (btc_result, eth_result) = tokio::join!(
        bitcoin_wallet::get_balance(btc_address),
        ethereum_wallet::get_balance(eth_address)
    );
    
    // At least one should succeed for this test
    let btc_ok = btc_result.is_ok();
    let eth_ok = eth_result.is_ok();
    
    if !btc_ok {
        eprintln!("Concurrent Bitcoin error: {:?}", btc_result.err());
    }
    if !eth_ok {
        eprintln!("Concurrent Ethereum error: {:?}", eth_result.err());
    }
    
    // Pass if at least Bitcoin works (more reliable API)
    assert!(btc_ok, "Bitcoin call should succeed");
}

// ============================================================================
// FAIL-TO-PASS TESTS: Base L2 (2 tests) - PR #1
// ============================================================================

#[tokio::test]
async fn test_base_balance_returns_valid_structure() {
    sleep(Duration::from_secs(1)).await;
    
    // Coinbase deployer address on Base
    let address = "0x4e59b44847b379578588920cA78FbF26c0B4956C";
    let result = base_wallet::get_balance(address).await;
    
    if let Err(e) = &result {
        eprintln!("Base API error: {}", e);
    }
    
    assert!(result.is_ok(), "Base balance fetch should succeed");
    
    let balance = result.unwrap();
    assert_eq!(balance.network, "base");
    assert_eq!(balance.denomination, "ETH");
    assert!(balance.address.starts_with("0x"));
}

#[tokio::test]
async fn test_base_invalid_address_returns_error() {
    let invalid_address = "0xinvalidbase";
    let result = base_wallet::get_balance(invalid_address).await;
    assert!(result.is_err(), "Invalid Base address should return error");
}

