//! Wallet Balance Library
//!
//! This library provides functionality to check cryptocurrency wallet balances
//! across multiple blockchain networks.

pub mod bitcoin_wallet;
pub mod ethereum_wallet;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a wallet balance with amount and denomination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalletBalance {
    pub address: String,
    pub balance: String,
    pub network: String,
    pub denomination: String,
}

impl WalletBalance {
    /// Create a new WalletBalance instance
    pub fn new(address: String, balance: String, network: String, denomination: String) -> Self {
        Self {
            address,
            balance,
            network,
            denomination,
        }
    }
}

/// Network enum for supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Bitcoin,
    Ethereum,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Bitcoin => write!(f, "bitcoin"),
            Network::Ethereum => write!(f, "ethereum"),
        }
    }
}

impl std::str::FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bitcoin" | "btc" => Ok(Network::Bitcoin),
            "ethereum" | "eth" => Ok(Network::Ethereum),
            _ => Err(anyhow::anyhow!("Unsupported network: {}", s)),
        }
    }
}
