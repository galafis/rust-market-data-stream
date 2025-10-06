use crate::types::MarketDataMessage;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;

/// WebSocket client for market data streaming
pub struct MarketDataClient {
    url: String,
    broadcast_tx: broadcast::Sender<MarketDataMessage>,
    running: Arc<tokio::sync::Mutex<bool>>,
}

impl MarketDataClient {
    pub fn new(url: String, buffer_size: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(buffer_size);
        
        Self {
            url,
            broadcast_tx,
            running: Arc::new(tokio::sync::Mutex::new(false)),
        }
    }

    /// Subscribe to market data stream
    pub fn subscribe(&self) -> broadcast::Receiver<MarketDataMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Start streaming market data
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            warn!("Client already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Connecting to {}", self.url);

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        info!("Connected successfully");

        let (mut write, mut read) = ws_stream.split();
        let broadcast_tx = self.broadcast_tx.clone();
        let running = Arc::clone(&self.running);

        // Send subscription message
        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "channels": ["trades", "quotes", "orderbook"]
        });
        
        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .map_err(|e| ClientError::WebSocket(e.to_string()))?;

        // Spawn message processing task
        tokio::spawn(async move {
            while *running.lock().await {
                match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received message: {}", text);
                        
                        match serde_json::from_str::<MarketDataMessage>(&text) {
                            Ok(msg) => {
                                if let Err(e) = broadcast_tx.send(msg) {
                                    error!("Failed to broadcast message: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {} - {}", e, text);
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        debug!("Received ping, sending pong");
                        // Pong is handled automatically by tokio-tungstenite
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Connection closed by server");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("Stream ended");
                        break;
                    }
                    _ => {}
                }
            }
            
            info!("Message processing task stopped");
        });

        Ok(())
    }

    /// Stop streaming
    pub async fn stop(&self) {
        info!("Stopping client");
        let mut running = self.running.lock().await;
        *running = false;
    }

    /// Check if client is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = MarketDataClient::new("ws://localhost:8080".to_string(), 1000);
        assert!(!client.is_running().await);
    }

    #[tokio::test]
    async fn test_subscription() {
        let client = MarketDataClient::new("ws://localhost:8080".to_string(), 1000);
        let _receiver = client.subscribe();
        // Subscription should work even if not connected
    }
}
