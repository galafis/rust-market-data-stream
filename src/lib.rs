//! # Rust Market Data Stream
//!
//! A high-performance real-time market data streaming and processing engine built with Rust.
//!
//! ## Features
//!
//! - **WebSocket Client**: Async WebSocket client for real-time market data feeds
//! - **Multiple Data Types**: Support for trades, quotes, and order book snapshots
//! - **Market Statistics**: Real-time calculation of VWAP, high/low, volume
//! - **Broadcast Channels**: Efficient message distribution to multiple consumers
//! - **Error Handling**: Comprehensive error types and recovery mechanisms
//!
//! ## Example
//!
//! ```rust,no_run
//! use rust_market_data_stream::{MarketDataClient, MarketDataMessage};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = MarketDataClient::new(
//!         "ws://localhost:8080".to_string(),
//!         1000
//!     );
//!     
//!     let mut receiver = client.subscribe();
//!     client.start().await.unwrap();
//!     
//!     while let Ok(msg) = receiver.recv().await {
//!         match msg {
//!             MarketDataMessage::Trade(trade) => {
//!                 println!("Trade: {} @ {}", trade.symbol, trade.price);
//!             }
//!             MarketDataMessage::Quote(quote) => {
//!                 println!("Quote: {} - bid: {} ask: {}", 
//!                          quote.symbol, quote.bid_price, quote.ask_price);
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```

pub mod client;
pub mod types;

pub use client::{ClientError, MarketDataClient};
pub use types::{
    MarketDataMessage, MarketStats, OrderBookSnapshot, PriceLevel, Quote, Trade, TradeSide,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_calculations() {
        let quote = Quote {
            symbol: "BTCUSD".to_string(),
            bid_price: 50000.0,
            bid_size: 1.5,
            ask_price: 50100.0,
            ask_size: 2.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(quote.spread(), 100.0);
        assert_eq!(quote.mid_price(), 50050.0);
    }

    #[test]
    fn test_market_stats() {
        let mut stats = MarketStats::new("BTCUSD".to_string());
        
        let trade1 = Trade {
            symbol: "BTCUSD".to_string(),
            price: 50000.0,
            quantity: 1.0,
            side: TradeSide::Buy,
            timestamp: chrono::Utc::now(),
            trade_id: "1".to_string(),
        };
        
        stats.update_with_trade(&trade1);
        
        assert_eq!(stats.trade_count, 1);
        assert_eq!(stats.last_price, 50000.0);
        assert_eq!(stats.high, 50000.0);
        assert_eq!(stats.low, 50000.0);
    }
}
