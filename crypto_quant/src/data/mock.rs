use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Balance, Exchange, Kline, Order, OrderBook, OrderStatus, Position, Side, Ticker};

pub struct MockExchange {
    klines: Vec<Kline>,
    orders: HashMap<String, Order>,
    balances: HashMap<String, Balance>,
    current_price: f64,
}

impl MockExchange {
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        balances.insert(
            "USDT".to_string(),
            Balance {
                asset: "USDT".to_string(),
                free: 10000.0,
                locked: 0.0,
            },
        );
        balances.insert(
            "BTC".to_string(),
            Balance {
                asset: "BTC".to_string(),
                free: 0.0,
                locked: 0.0,
            },
        );

        Self {
            klines: Self::generate_mock_klines(),
            orders: HashMap::new(),
            balances,
            current_price: 50000.0,
        }
    }

    fn generate_mock_klines() -> Vec<Kline> {
        let mut klines = Vec::new();
        let mut price = 50000.0;
        let now = Utc::now();

        for i in (0..1000).rev() {
            let timestamp = now - Duration::hours(i);
            let change = (rand::random::<f64>() - 0.5) * 0.02;
            price *= 1.0 + change;

            let open = price;
            let close = price * (1.0 + (rand::random::<f64>() - 0.5) * 0.01);
            let high = open.max(close) * (1.0 + rand::random::<f64>() * 0.005);
            let low = open.min(close) * (1.0 - rand::random::<f64>() * 0.005);
            let volume = 100.0 + rand::random::<f64>() * 900.0;

            klines.push(Kline {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
                quote_volume: volume * close,
                trades: (volume / 10.0) as u64,
            });

            price = close;
        }

        klines
    }

    pub fn set_price(&mut self, price: f64) {
        self.current_price = price;
    }
}

#[async_trait]
impl Exchange for MockExchange {
    async fn get_klines(
        &self,
        _symbol: &str,
        _timeframe: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<Kline>> {
        let mut klines = self.klines.clone();

        if let Some(start) = start_time {
            klines.retain(|k| k.timestamp >= start);
        }
        if let Some(end) = end_time {
            klines.retain(|k| k.timestamp <= end);
        }

        if let Some(lim) = limit {
            let start_idx = klines.len().saturating_sub(lim);
            klines = klines.split_off(start_idx);
        }

        Ok(klines)
    }

    async fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            price: self.current_price,
            timestamp: Utc::now(),
        })
    }

    async fn get_order_book(&self, symbol: &str, depth: Option<usize>) -> Result<OrderBook> {
        let depth = depth.unwrap_or(10);
        let price = self.current_price;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for i in 0..depth {
            let bid_price = price * (1.0 - (i as f64 + 1.0) * 0.001);
            let ask_price = price * (1.0 + (i as f64 + 1.0) * 0.001);
            let quantity = 1.0 + rand::random::<f64>() * 10.0;

            bids.push((bid_price, quantity));
            asks.push((ask_price, quantity));
        }

        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn place_order(&self, mut order: Order) -> Result<Order> {
        order.id = Uuid::new_v4().to_string();
        order.status = OrderStatus::Filled;
        order.filled_quantity = order.quantity;
        order.avg_price = self.current_price;
        Ok(order)
    }

    async fn cancel_order(&self, _symbol: &str, order_id: &str) -> Result<bool> {
        Ok(true)
    }

    async fn get_order(&self, _symbol: &str, order_id: &str) -> Result<Option<Order>> {
        Ok(self.orders.get(order_id).cloned())
    }

    async fn get_balance(&self) -> Result<HashMap<String, Balance>> {
        Ok(self.balances.clone())
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        Ok(vec![])
    }
}
