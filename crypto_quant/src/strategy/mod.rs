use crate::data::{Kline, Order, OrderType, Side};
use crate::indicator::TechnicalIndicators;
use anyhow::Result;
use chrono::{DateTime, Utc};

pub mod dual_ma;
pub mod rsi_strategy;

pub use dual_ma::DualMaStrategy;
pub use rsi_strategy::RsiStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone)]
pub struct StrategyContext {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub current_price: f64,
    pub position_size: f64,
    pub unrealized_pnl: f64,
    pub balance: f64,
}

pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;

    fn on_data(&mut self, klines: &[Kline]) -> Result<Signal>;

    fn generate_order(&self, signal: Signal, context: &StrategyContext) -> Option<Order> {
        match signal {
            Signal::Buy => {
                if context.position_size > 0.0 {
                    return None;
                }
                Some(Order {
                    id: String::new(),
                    symbol: context.symbol.clone(),
                    side: Side::Buy,
                    order_type: OrderType::Market,
                    quantity: self.calculate_position_size(context),
                    filled_quantity: 0.0,
                    avg_price: 0.0,
                    status: crate::data::OrderStatus::Pending,
                    timestamp: Utc::now(),
                })
            }
            Signal::Sell => {
                if context.position_size <= 0.0 {
                    return None;
                }
                Some(Order {
                    id: String::new(),
                    symbol: context.symbol.clone(),
                    side: Side::Sell,
                    order_type: OrderType::Market,
                    quantity: context.position_size,
                    filled_quantity: 0.0,
                    avg_price: 0.0,
                    status: crate::data::OrderStatus::Pending,
                    timestamp: Utc::now(),
                })
            }
            Signal::Hold => None,
        }
    }

    fn calculate_position_size(&self, context: &StrategyContext) -> f64 {
        let risk_per_trade = 0.02;
        let position_value = context.balance * risk_per_trade;
        position_value / context.current_price
    }

    fn get_indicators(&self) -> &TechnicalIndicators {
        static INDICATORS: std::sync::OnceLock<TechnicalIndicators> = std::sync::OnceLock::new();
        INDICATORS.get_or_init(|| TechnicalIndicators)
    }
}
