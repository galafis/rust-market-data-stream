use rust_market_data_stream::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    println!("=== Market Data Stream Example ===\n");
    
    // Create client
    let client = MarketDataClient::new("wss://stream.binance.com:9443/ws/btcusdt@trade".to_string());
    
    println!("Connecting to Binance WebSocket...");
    
    // Connect
    client.connect().await?;
    
    println!("Connected! Streaming market data...\n");
    
    // Subscribe to trades
    client.subscribe(vec!["btcusdt@trade".to_string()]).await?;
    
    // Receive and process messages
    let mut count = 0;
    while let Some(message) = client.receive().await {
        match message {
            MarketDataMessage::Trade(trade) => {
                println!(
                    "Trade: {} {} @ {} ({})",
                    trade.symbol,
                    trade.quantity,
                    trade.price,
                    trade.timestamp
                );
                
                count += 1;
                if count >= 10 {
                    println!("\nReceived 10 trades, disconnecting...");
                    break;
                }
            }
            MarketDataMessage::Quote(quote) => {
                println!(
                    "Quote: {} - Bid: {} @ {} | Ask: {} @ {}",
                    quote.symbol,
                    quote.bid_quantity,
                    quote.bid_price,
                    quote.ask_quantity,
                    quote.ask_price
                );
            }
            MarketDataMessage::OrderBook(snapshot) => {
                println!(
                    "OrderBook: {} - {} bids, {} asks",
                    snapshot.symbol,
                    snapshot.bids.len(),
                    snapshot.asks.len()
                );
            }
        }
    }
    
    // Disconnect
    client.disconnect().await?;
    println!("Disconnected.");
    
    // Get statistics
    let stats = client.get_statistics();
    println!("\n=== Statistics ===");
    println!("Messages received: {}", stats.messages_received);
    println!("Bytes received: {}", stats.bytes_received);
    println!("VWAP: {:.2}", stats.vwap);
    println!("Volume: {:.2}", stats.volume);
    println!("High: {:.2}", stats.high);
    println!("Low: {:.2}", stats.low);
    
    Ok(())
}
