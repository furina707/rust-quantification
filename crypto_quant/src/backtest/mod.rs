use crate::config::AppConfig;
use crate::data::{Exchange, Order, OrderStatus, Side};
use crate::risk::RiskManager;
use crate::strategy::Strategy;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub window_size: usize,
    pub symbol: String,
    pub timeframe: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_capital: f64,
}

#[derive(Debug)]
pub struct BacktestReport {
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_return_pct: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_ratio: f64,
    pub trades: Vec<TradeRecord>,
}

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
}

pub struct BacktestEngine {
    config: BacktestConfig,
    exchange: Arc<dyn Exchange>,
    strategy: Box<dyn Strategy>,
    risk_manager: RiskManager,
    balance: f64,
    position_size: f64,
    entry_price: f64,
    position_side: Option<Side>,
    trades: Vec<TradeRecord>,
    equity_curve: Vec<f64>,
}

impl BacktestEngine {
    pub fn new(
        config: BacktestConfig,
        exchange: Arc<dyn Exchange>,
        strategy: Box<dyn Strategy>,
        risk_manager: RiskManager,
    ) -> Self {
        let initial_capital = config.initial_capital;
        Self {
            config,
            exchange,
            strategy,
            risk_manager,
            balance: initial_capital,
            position_size: 0.0,
            entry_price: 0.0,
            position_side: None,
            trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> Result<BacktestReport> {
        let klines = self.exchange.get_klines(
            &self.config.symbol,
            &self.config.timeframe,
            Some(self.config.start_time),
            Some(self.config.end_time),
            None,
        ).await?;

        let window_size = self.config.window_size;

        for i in window_size..klines.len() {
            let current_kline = &klines[i];
            let window = &klines[i - window_size..i];

            let signal = self.strategy.on_data(window)?;

            if signal == crate::strategy::Signal::Buy && self.position_side.is_none() {
                let quantity = self.balance / current_kline.close;
                if self.risk_manager.check_position_size(quantity, current_kline.close) {
                    self.position_side = Some(Side::Buy);
                    self.position_size = quantity;
                    self.entry_price = current_kline.close;
                    self.balance -= quantity * current_kline.close;
                }
            } else if signal == crate::strategy::Signal::Sell && self.position_side.is_some() {
                let exit_price = current_kline.close;
                let pnl = (exit_price - self.entry_price) * self.position_size;
                let pnl_pct = (exit_price - self.entry_price) / self.entry_price * 100.0;

                let entry_time = window.last().map(|k| k.timestamp).unwrap_or(Utc::now());

                self.trades.push(TradeRecord {
                    entry_time,
                    exit_time: current_kline.timestamp,
                    entry_price: self.entry_price,
                    exit_price,
                    quantity: self.position_size,
                    pnl,
                    pnl_pct,
                });

                self.balance += self.position_size * exit_price;
                self.position_side = None;
                self.position_size = 0.0;
                self.entry_price = 0.0;
            }

            if self.position_side.is_some() {
                let unrealized_pnl = (current_kline.close - self.entry_price) * self.position_size;
                let current_equity = self.balance + self.position_size * current_kline.close;

                if self.risk_manager.check_stop_loss(self.entry_price, current_kline.close, Side::Buy)
                    || self.risk_manager.check_take_profit(self.entry_price, current_kline.close, Side::Buy)
                {
                    let exit_price = current_kline.close;
                    let pnl = (exit_price - self.entry_price) * self.position_size;
                    let pnl_pct = (exit_price - self.entry_price) / self.entry_price * 100.0;

                    let entry_time = window.last().map(|k| k.timestamp).unwrap_or(Utc::now());

                    self.trades.push(TradeRecord {
                        entry_time,
                        exit_time: current_kline.timestamp,
                        entry_price: self.entry_price,
                        exit_price,
                        quantity: self.position_size,
                        pnl,
                        pnl_pct,
                    });

                    self.balance += self.position_size * exit_price;
                    self.position_side = None;
                    self.position_size = 0.0;
                    self.entry_price = 0.0;
                } else {
                    self.equity_curve.push(current_equity);
                }
            } else {
                self.equity_curve.push(self.balance);
            }
        }

        let final_capital = self.balance;
        let total_return_pct = (final_capital - self.config.initial_capital) / self.config.initial_capital * 100.0;

        let winning_trades = self.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = self.trades.iter().filter(|t| t.pnl <= 0.0).count();
        let total_trades = self.trades.len();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let max_drawdown_pct = self.calculate_max_drawdown();
        let sharpe_ratio = self.calculate_sharpe_ratio();

        Ok(BacktestReport {
            initial_capital: self.config.initial_capital,
            final_capital,
            total_return_pct,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            max_drawdown_pct,
            sharpe_ratio,
            trades: self.trades.clone(),
        })
    }

    fn calculate_max_drawdown(&self) -> f64 {
        if self.equity_curve.is_empty() {
            return 0.0;
        }

        let mut peak = self.equity_curve[0];
        let mut max_drawdown = 0.0;

        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        max_drawdown
    }

    fn calculate_sharpe_ratio(&self) -> f64 {
        if self.equity_curve.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = self.equity_curve
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        let annualized_return = mean_return * 365.0;
        let annualized_std = std_dev * (365.0_f64).sqrt();

        annualized_return / annualized_std
    }
}
