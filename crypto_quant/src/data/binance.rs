use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

use super::{Balance, Exchange, Kline, Order, OrderBook, Position, Side, Ticker};

pub struct BinanceExchange {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl BinanceExchange {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.binance.com".to_string(),
            api_key: None,
            api_secret: None,
        }
    }

    pub fn with_credentials(mut self, api_key: String, api_secret: String) -> Self {
        self.api_key = Some(api_key);
        self.api_secret = Some(api_secret);
        self
    }

    fn timeframe_to_interval(&self, timeframe: &str) -> &str {
        match timeframe {
            "1m" => "1m",
            "5m" => "5m",
            "15m" => "15m",
            "30m" => "30m",
            "1h" => "1h",
            "4h" => "4h",
            "1d" => "1d",
            "1w" => "1w",
            _ => "1h",
        }
    }
}

#[derive(Debug, Deserialize)]
struct BinanceKline {
    #[serde(rename = "0")]
    open_time: i64,
    #[serde(rename = "1")]
    open: String,
    #[serde(rename = "2")]
    high: String,
    #[serde(rename = "3")]
    low: String,
    #[serde(rename = "4")]
    close: String,
    #[serde(rename = "5")]
    volume: String,
    #[serde(rename = "6")]
    close_time: i64,
    #[serde(rename = "7")]
    quote_volume: String,
    #[serde(rename = "8")]
    trades: u64,
}

#[derive(Debug, Deserialize)]
struct BinanceTicker {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
}

#[derive(Debug, Deserialize)]
struct BinanceOrderBook {
    bids: Vec<Vec<String>>,
    asks: Vec<Vec<String>>,
}

#[async_trait]
impl Exchange for BinanceExchange {
    async fn get_klines(
        &self,
        symbol: &str,
        timeframe: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<Kline>> {
        let interval = self.timeframe_to_interval(timeframe);
        let symbol = symbol.to_uppercase();

        let mut url = format!(
            "{}/api/v3/klines?symbol={}&interval={}",
            self.base_url, symbol, interval
        );

        if let Some(start) = start_time {
            url.push_str(&format!("&startTime={}", start.timestamp_millis()));
        }
        if let Some(end) = end_time {
            url.push_str(&format!("&endTime={}", end.timestamp_millis()));
        }
        if let Some(lim) = limit {
            url.push_str(&format!("&limit={}", lim.min(1000)));
        }

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch klines: {}",
                response.text().await?
            ));
        }

        let binance_klines: Vec<BinanceKline> = response.json().await?;

        let klines: Vec<Kline> = binance_klines
            .into_iter()
            .map(|k| Kline {
                timestamp: Utc.timestamp_millis_opt(k.open_time).unwrap(),
                open: k.open.parse().unwrap_or(0.0),
                high: k.high.parse().unwrap_or(0.0),
                low: k.low.parse().unwrap_or(0.0),
                close: k.close.parse().unwrap_or(0.0),
                volume: k.volume.parse().unwrap_or(0.0),
                quote_volume: k.quote_volume.parse().unwrap_or(0.0),
                trades: k.trades,
            })
            .collect();

        Ok(klines)
    }

    async fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        let symbol = symbol.to_uppercase();
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.base_url, symbol);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch ticker: {}", response.text().await?));
        }

        let ticker: BinanceTicker = response.json().await?;

        Ok(Ticker {
            symbol: ticker.symbol,
            price: ticker.last_price.parse().unwrap_or(0.0),
            timestamp: Utc::now(),
        })
    }

    async fn get_order_book(&self, symbol: &str, depth: Option<usize>) -> Result<OrderBook> {
        let symbol = symbol.to_uppercase();
        let limit = depth.unwrap_or(100).min(1000);
        let url = format!(
            "{}/api/v3/depth?symbol={}&limit={}",
            self.base_url, symbol, limit
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch order book: {}",
                response.text().await?
            ));
        }

        let order_book: BinanceOrderBook = response.json().await?;

        let bids: Vec<(f64, f64)> = order_book
            .bids
            .into_iter()
            .filter_map(|b| {
                if b.len() >= 2 {
                    Some((b[0].parse().ok()?, b[1].parse().ok()?))
                } else {
                    None
                }
            })
            .collect();

        let asks: Vec<(f64, f64)> = order_book
            .asks
            .into_iter()
            .filter_map(|a| {
                if a.len() >= 2 {
                    Some((a[0].parse().ok()?, a[1].parse().ok()?))
                } else {
                    None
                }
            })
            .collect();

        Ok(OrderBook {
            symbol,
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn place_order(&self, _order: Order) -> Result<Order> {
        Err(anyhow!("Not implemented for public API"))
    }

    async fn cancel_order(&self, _symbol: &str, _order_id: &str) -> Result<bool> {
        Err(anyhow!("Not implemented for public API"))
    }

    async fn get_order(&self, _symbol: &str, _order_id: &str) -> Result<Option<Order>> {
        Err(anyhow!("Not implemented for public API"))
    }

    async fn get_balance(&self) -> Result<HashMap<String, Balance>> {
        Err(anyhow!("Not implemented for public API"))
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        Ok(vec![])
    }
}
