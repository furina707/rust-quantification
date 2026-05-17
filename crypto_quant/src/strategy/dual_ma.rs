use anyhow::Result;

use crate::data::Kline;
use crate::indicator::TechnicalIndicators;
use crate::strategy::{Signal, Strategy};

pub struct DualMaStrategy {
    name: String,
    fast_period: usize,
    slow_period: usize,
    prev_fast_ma: Option<f64>,
    prev_slow_ma: Option<f64>,
}

impl DualMaStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            name: format!("DualMA({},{})", fast_period, slow_period),
            fast_period,
            slow_period,
            prev_fast_ma: None,
            prev_slow_ma: None,
        }
    }
}

impl Strategy for DualMaStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_data(&mut self, klines: &[Kline]) -> Result<Signal> {
        if klines.len() < self.slow_period + 1 {
            return Ok(Signal::Hold);
        }

        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();

        let fast_ma = TechnicalIndicators::sma(&closes, self.fast_period);
        let slow_ma = TechnicalIndicators::sma(&closes, self.slow_period);

        let current_idx = closes.len() - 1;
        let prev_idx = closes.len() - 2;

        let current_fast = fast_ma[current_idx];
        let current_slow = slow_ma[current_idx];
        let prev_fast = fast_ma[prev_idx];
        let prev_slow = slow_ma[prev_idx];

        if let (Some(cf), Some(cs), Some(pf), Some(ps)) =
            (current_fast, current_slow, prev_fast, prev_slow)
        {
            if pf <= ps && cf > cs {
                return Ok(Signal::Buy);
            }

            if pf >= ps && cf < cs {
                return Ok(Signal::Sell);
            }
        }

        Ok(Signal::Hold)
    }
}
