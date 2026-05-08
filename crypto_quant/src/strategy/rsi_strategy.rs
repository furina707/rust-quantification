use anyhow::Result;

use crate::data::Kline;
use crate::indicator::TechnicalIndicators;
use crate::strategy::{Signal, Strategy};

pub struct RsiStrategy {
    name: String,
    period: usize,
    overbought: f64,
    oversold: f64,
}

impl RsiStrategy {
    pub fn new(period: usize, overbought: f64, oversold: f64) -> Self {
        Self {
            name: format!("RSI({})", period),
            period,
            overbought,
            oversold,
        }
    }

    pub fn default() -> Self {
        Self::new(14, 70.0, 30.0)
    }
}

impl Strategy for RsiStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_data(&mut self, klines: &[Kline]) -> Result<Signal> {
        if klines.len() < self.period + 1 {
            return Ok(Signal::Hold);
        }

        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
        let indicators = TechnicalIndicators;

        let rsi_values = indicators.rsi(&closes, self.period);

        let current_idx = closes.len() - 1;
        let prev_idx = closes.len() - 2;

        let current_rsi = rsi_values[current_idx];
        let prev_rsi = rsi_values[prev_idx];

        if let (Some(cr), Some(pr)) = (current_rsi, prev_rsi) {
            if pr >= self.oversold && cr < self.oversold {
                return Ok(Signal::Buy);
            }

            if pr <= self.overbought && cr > self.overbought {
                return Ok(Signal::Sell);
            }
        }

        Ok(Signal::Hold)
    }
}
