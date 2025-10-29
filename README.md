# Wallet Balance CLI

A command-line tool to check cryptocurrency wallet balances for Bitcoin and Ethereum networks.

## Features

- ✅ Bitcoin wallet balance checking (via Blockchain.com API)
- ✅ Ethereum wallet balance checking (via Public RPC endpoints)
- 🔜 Ethereum L2 support (Optimism, Arbitrum, Base) - Coming in PRs

## Installation

### Prerequisites

- Rust 1.70+ installed
- Internet connection for API calls

### Build from source

```bash
cargo build --release
```

The binary will be available at `target/release/wallet-balance`

## Usage

### Check Bitcoin Balance

```bash
cargo run -- --network bitcoin --address 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
```

### Check Ethereum Balance

```bash
cargo run -- --network ethereum --address 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
```

### CLI Options

```
wallet-balance [OPTIONS]

Options:
  -n, --network <NETWORK>    Network to check (bitcoin, ethereum)
  -a, --address <ADDRESS>    Wallet address to check
  -h, --help                 Print help
  -V, --version              Print version
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test tests

# Run with output
cargo test -- --nocapture
```

### Test-Driven Development (TDD)

This project follows TDD principles:

1. **Base commit**: Contains 10 pass-to-pass tests for Bitcoin and Ethereum
2. **Future PRs**: Each L2 network (Optimism, Arbitrum, Base) will have fail-to-pass tests

### Adding New Networks (L2s)

To add a new network support:

1. Write failing test in `tests/tests.rs`
2. Create network module in `src/` (e.g., `optimism_wallet.rs`)
3. Add network to `src/lib.rs` exports
4. Implement balance checking logic
5. Update CLI in `src/main.rs` to include new network
6. Ensure tests pass

## Project Structure

```
wallet-balance-cli/
├── Cargo.toml                  # Project dependencies
├── README.md                   # This file
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library exports
│   ├── bitcoin_wallet.rs       # Bitcoin implementation
│   └── ethereum_wallet.rs      # Ethereum implementation
    |--- base_wallet.rs          # Base Implementation 
    |--- arbitrum_wallet.rs    # Arbitrum Implementation.
└── tests/
    └── tests.rs                # 10 pass-to-pass tests
```

## API Endpoints Used

- **Bitcoin**: Blockchain.com API (https://blockchain.info)
- **Ethereum**: Public RPC (https://eth.public-rpc.com)

## Examples

### Satoshi's Bitcoin Address
```bash
cargo run -- -n bitcoin -a 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
# Output: Balance: 0.00000000 BTC
```

### Vitalik's Ethereum Address
```bash
cargo run -- -n ethereum -a 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
# Output: Balance: 2435.123456789 ETH
```

## Roadmap

- [x] Bitcoin balance checking
- [x] Ethereum balance checking
- [ ] Optimism L2 support (PR #1)
- [x] Arbitrum L2 support (PR #2)
- [x] Base L2 support (PR #3)
- [ ] ERC-20 token balance support
- [ ] Transaction history

## Contributing

1. Fork the repository
2. Create feature branch with TDD approach
3. Write failing tests first
4. Implement minimum code to pass tests
5. Submit PR with test evidence

## License

MIT License
