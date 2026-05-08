mod backtest;
mod config;
mod data;
mod indicator;
mod risk;
mod strategy;

use anyhow::Result;
use log::info;
use std::sync::Arc;

use crate::backtest::{BacktestEngine, BacktestConfig};
use crate::config::AppConfig;
use crate::data::{Exchange, MockExchange};
use crate::risk::RiskManager;
use crate::strategy::{DualMaStrategy, Strategy};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("启动虚拟币量化交易系统...");
    
    let config = AppConfig::load()?;
    
    let exchange = Arc::new(MockExchange::new());
    
    let strategy = DualMaStrategy::new(5, 20);
    
    let risk_manager = RiskManager::new(
        config.max_position_size,
        config.stop_loss_pct,
        config.take_profit_pct,
    );
    
    let backtest_config = BacktestConfig {
        symbol: config.symbol.clone(),
        timeframe: config.timeframe.clone(),
        start_time: config.start_time,
        end_time: config.end_time,
        initial_capital: config.initial_capital,
    };
    
    let mut engine = BacktestEngine::new(
        backtest_config,
        exchange,
        Box::new(strategy),
        risk_manager,
    );
    
    info!("开始回测...");
    let report = engine.run().await?;
    
    println!("\n========== 回测报告 ==========");
    println!("交易对: {}", config.symbol);
    println!("时间周期: {}", config.timeframe);
    println!("初始资金: ${:.2}", report.initial_capital);
    println!("最终资金: ${:.2}", report.final_capital);
    println!("总收益率: {:.2}%", report.total_return_pct);
    println!("总交易次数: {}", report.total_trades);
    println!("盈利交易: {}", report.winning_trades);
    println!("亏损交易: {}", report.losing_trades);
    println!("胜率: {:.2}%", report.win_rate * 100.0);
    println!("最大回撤: {:.2}%", report.max_drawdown_pct);
    println!("夏普比率: {:.2}", report.sharpe_ratio);
    
    Ok(())
}
