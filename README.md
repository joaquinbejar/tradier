# Tradier: Rust Library for Tradier Broker API

[![Dual License](https://img.shields.io/badge/license-MIT%20and%20Apache%202.0-blue)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/tradier.svg)](https://crates.io/crates/tradier)
[![Downloads](https://img.shields.io/crates/d/tradier.svg)](https://crates.io/crates/tradier)
[![Stars](https://img.shields.io/github/stars/yourusername/TradierRust.svg)](https://github.com/yourusername/TradierRust/stargazers)

[![Build Status](https://img.shields.io/github/workflow/status/yourusername/TradierRust/CI)](https://github.com/yourusername/TradierRust/actions)
[![Coverage](https://img.shields.io/codecov/c/github/yourusername/TradierRust)](https://codecov.io/gh/yourusername/TradierRust)
[![Dependencies](https://img.shields.io/librariesio/github/yourusername/TradierRust)](https://libraries.io/github/yourusername/TradierRust)

## Table of Contents
1. [Introduction](#introduction)
2. [Features](#features)
3. [Project Structure](#project-structure)
4. [Setup Instructions](#setup-instructions)
5. [Library Usage](#library-usage)
6. [Usage Examples](#usage-examples)
7. [Testing](#testing)
8. [Contribution and Contact](#contribution-and-contact)

## Introduction

TradierRust is a comprehensive Rust library for managing trades and market data using the Tradier broker API. This powerful toolkit enables developers, traders, and financial applications to:

- Execute trades efficiently
- Retrieve real-time quotes
- Manage portfolios
- Access historical market data

The library leverages Rust's performance and concurrency advantages, making it suitable for high-frequency trading applications and data-intensive financial processing.

## Features

1. **Trade Execution**: Implement order placement, modification, and cancellation.
2. **Real-time Market Data**: Access live quotes, order book data, and trade information.
3. **Portfolio Management**: Retrieve account information, positions, and performance metrics.
4. **Historical Data**: Fetch and analyze historical price and volume data.
5. **Streaming Data**: Utilize WebSocket connections for real-time data feeds.
6. **Authentication**: Securely manage API keys and authentication tokens.
7. **Error Handling**: Robust error handling and logging for reliability.
8. **Rate Limiting**: Implement rate limiting to comply with API usage restrictions.
9. **Concurrent Processing**: Leverage Rust's async capabilities for efficient data handling.
10. **Data Serialization**: Use Serde for efficient JSON parsing and serialization.

## Project Structure

The project is structured as follows:

1. **API Client** (`client.rs`): Core implementation of the Tradier API client.
2. **Authentication** (`auth.rs`): Handles API authentication and token management.
3. **Trade Execution** (`trade/`):
   - `order.rs`: Structures and methods for order management.
   - `execution.rs`: Handles trade execution and order status updates.
4. **Market Data** (`market_data/`):
   - `quote.rs`: Real-time and delayed quote retrieval.
   - `historical.rs`: Historical price and volume data fetching.
   - `streaming.rs`: WebSocket implementation for live data streaming.
5. **Account Management** (`account/`):
   - `portfolio.rs`: Portfolio and position management.
   - `balance.rs`: Account balance and margin information.
6. **Data Models** (`models/`): Rust structures representing various API responses.
7. **Error Handling** (`error.rs`): Custom error types and error handling utilities.
8. **Utilities** (`utils/`):
   - `rate_limiter.rs`: Implements rate limiting for API requests.
   - `serialization.rs`: Custom serialization and deserialization functions.

## Setup Instructions

1. Add TradierRust to your `Cargo.toml`:
   ```toml
   [dependencies]
   tradier = "0.1.0"
   ```

2. Set up your Tradier API credentials:
   - Create a `.env` file in your project root:
     ```
     TRADIER_API_KEY=your_api_key_here
     TRADIER_ACCOUNT_ID=your_account_id_here
     ```

3. Build your project:
   ```shell
   cargo build
   ```

## Library Usage

To use the library in your project:

```rust
use tradier::TradierClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TradierClient::new()?;
    // Use the client to interact with the Tradier API
    Ok(())
}
```

## Usage Examples

Here are some examples of how to use the library:

```rust
use tradier::{TradierClient, Order, Quote};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TradierClient::new()?;

    // Fetch a quote
    let quote: Quote = client.get_quote("AAPL").await?;
    println!("AAPL quote: {:?}", quote);

    // Place an order
    let order = Order::market("AAPL", 10, tradier::Side::Buy);
    let result = client.place_order(order).await?;
    println!("Order placed: {:?}", result);

    Ok(())
}
```

## Testing

To run unit tests:
```shell
make test
```

To run tests with coverage:
```shell
make coverage
```

## Contribution and Contact

We welcome contributions to this project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure that the project still builds and all tests pass.
4. Commit your changes and push your branch to your forked repository.
5. Submit a pull request to the main repository.

If you have any questions, issues, or would like to provide feedback, please feel free to contact the project maintainer:

**Joaquín Béjar García**
- Email: jb@taunais.com
- GitHub: [joaquinbejar](https://github.com/joaquinbejar)

We appreciate your interest and look forward to your contributions!
