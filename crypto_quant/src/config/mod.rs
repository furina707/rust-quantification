use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub symbol: String,
    pub timeframe: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_capital: f64,
    pub max_position_size: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub strategy: StrategyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub fast_period: Option<usize>,
    pub slow_period: Option<usize>,
    pub rsi_period: Option<usize>,
    pub overbought: Option<f64>,
    pub oversold: Option<f64>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        if let Ok(config_str) = fs::read_to_string("config.toml") {
            toml::from_str(&config_str).map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write("config.toml", config_str)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            start_time: now - Duration::days(30),
            end_time: now,
            initial_capital: 10000.0,
            max_position_size: 1000.0,
            stop_loss_pct: 2.0,
            take_profit_pct: 5.0,
            strategy: StrategyConfig {
                name: "dual_ma".to_string(),
                fast_period: Some(5),
                slow_period: Some(20),
                rsi_period: None,
                overbought: None,
                oversold: None,
            },
        }
    }
}
