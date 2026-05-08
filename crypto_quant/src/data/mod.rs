use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod binance;
pub mod mock;

pub use binance::BinanceExchange;
pub use mock::MockExchange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub trades: u64,
}

#[derive(Debug, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit(f64),
    StopLoss(f64),
    TakeProfit(f64),
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub avg_price: f64,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: Side,
    pub quantity: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[async_trait]
pub trait Exchange: Send + Sync {
    async fn get_klines(
        &self,
        symbol: &str,
        timeframe: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<Kline>>;

    async fn get_ticker(&self, symbol: &str) -> Result<Ticker>;

    async fn get_order_book(&self, symbol: &str, depth: Option<usize>) -> Result<OrderBook>;

    async fn place_order(&self, order: Order) -> Result<Order>;

    async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<bool>;

    async fn get_order(&self, symbol: &str, order_id: &str) -> Result<Option<Order>>;

    async fn get_balance(&self) -> Result<HashMap<String, Balance>>;

    async fn get_positions(&self) -> Result<Vec<Position>>;
}
