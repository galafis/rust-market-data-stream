use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Market data message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarketDataMessage {
    Trade(Trade),
    Quote(Quote),
    OrderBook(OrderBookSnapshot),
    Heartbeat,
}

/// Trade tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: TradeSide,
    pub timestamp: DateTime<Utc>,
    pub trade_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Quote (BBO - Best Bid/Offer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
    pub timestamp: DateTime<Utc>,
}

impl Quote {
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    pub fn mid_price(&self) -> f64 {
        (self.bid_price + self.ask_price) / 2.0
    }
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
    pub num_orders: u32,
}

/// Full order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

impl OrderBookSnapshot {
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / 2.0),
            _ => None,
        }
    }

    pub fn total_bid_volume(&self) -> f64 {
        self.bids.iter().map(|level| level.size).sum()
    }

    pub fn total_ask_volume(&self) -> f64 {
        self.asks.iter().map(|level| level.size).sum()
    }
}

/// Market statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketStats {
    pub symbol: String,
    pub trade_count: u64,
    pub total_volume: f64,
    pub vwap: f64,
    pub high: f64,
    pub low: f64,
    pub last_price: f64,
    pub last_update: Option<DateTime<Utc>>,
}

impl MarketStats {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            trade_count: 0,
            total_volume: 0.0,
            vwap: 0.0,
            high: f64::MIN,
            low: f64::MAX,
            last_price: 0.0,
            last_update: None,
        }
    }

    pub fn update_with_trade(&mut self, trade: &Trade) {
        self.trade_count += 1;
        self.total_volume += trade.quantity;
        
        // Update VWAP
        let prev_total = self.vwap * (self.total_volume - trade.quantity);
        let new_total = prev_total + (trade.price * trade.quantity);
        self.vwap = new_total / self.total_volume;
        
        // Update high/low
        if trade.price > self.high {
            self.high = trade.price;
        }
        if trade.price < self.low {
            self.low = trade.price;
        }
        
        self.last_price = trade.price;
        self.last_update = Some(trade.timestamp);
    }
}
