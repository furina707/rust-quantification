# 加密货币量化交易回测系统

一个基于 Rust 语言开发的加密货币量化交易回测框架，支持多种技术指标和交易策略的策略回测与性能评估。

## 功能特性

- **多交易所支持**：内置 Binance 交易所适配器和模拟交易所
- **技术指标库**：支持 SMA、EMA、RSI、MACD、布林带、ATR 等常用技术指标
- **多种交易策略**：双均线交叉策略、RSI 超买超卖策略
- **风险管理**：内置止损止盈机制、仓位管理
- **回测引擎**：完整的回测框架，支持性能指标计算
- **异步架构**：基于 Tokio 异步运行时，支持高并发

## 技术栈

- **语言**：Rust 2021 Edition
- **异步运行时**：Tokio
- **HTTP 客户端**：Reqwest
- **序列化**：Serde
- **时间处理**：Chrono

## 项目结构

```
crypto_quant/
├── Cargo.toml           # 项目配置和依赖
├── src/
│   ├── main.rs          # 应用入口
│   ├── backtest/        # 回测引擎
│   │   └── mod.rs
│   ├── config/          # 配置管理
│   │   └── mod.rs
│   ├── data/            # 数据层
│   │   ├── mod.rs       # 数据模型定义
│   │   ├── binance.rs   # Binance 交易所适配器
│   │   └── mock.rs      # 模拟交易所
│   ├── indicator/        # 技术指标
│   │   └── mod.rs
│   ├── risk/            # 风险管理
│   │   └── mod.rs
│   └── strategy/        # 交易策略
│       ├── mod.rs       # 策略接口
│       ├── dual_ma.rs   # 双均线策略
│       └── rsi_strategy.rs  # RSI 策略
```

## 安装与运行

### 环境要求

- Rust 1.70.0 或更高版本
- Cargo 包管理器

### 构建项目

```bash
cd crypto_quant
cargo build --release
```

### 运行回测

```bash
cargo run --release
```

## 配置说明

项目支持通过 `config.toml` 文件进行配置，默认配置如下：

```toml
symbol = "BTCUSDT"           # 交易对
timeframe = "1h"             # 时间周期 (1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w)
start_time = "2024-01-01T00:00:00Z"
end_time = "2024-12-31T23:59:59Z"
initial_capital = 10000.0    # 初始资金
max_position_size = 1000.0   # 最大仓位
stop_loss_pct = 2.0          # 止损比例 (%)
take_profit_pct = 5.0        # 止盈比例 (%)

[strategy]
name = "dual_ma"             # 策略名称
fast_period = 5              # 快速均线周期
slow_period = 20             # 慢速均线周期
```

## 交易策略

### 双均线交叉策略 (Dual MA)

基于两条不同周期的移动平均线的交叉来判断买卖信号：

- **买入信号**：快速均线上穿慢速均线
- **卖出信号**：快速均线下穿慢速均线

参数配置：
- `fast_period`：快速均线周期（默认 5）
- `slow_period`：慢速均线周期（默认 20）

### RSI 策略

基于相对强弱指数（RSI）的超买超卖区间进行交易：

- **买入信号**：RSI 从超卖区上穿（RSI < 30）
- **卖出信号**：RSI 从超买区下穿（RSI > 70）

参数配置：
- `period`：RSI 计算周期（默认 14）
- `overbought`：超买阈值（默认 70）
- `oversold`：超卖阈值（默认 30）

## 回测报告

回测完成后，系统会输出详细的回测报告，包括：

| 指标 | 说明 |
|------|------|
| 交易对 | 交易的加密货币对 |
| 时间周期 | K 线时间周期 |
| 初始资金 | 回测起始资金 |
| 最终资金 | 回测结束资金 |
| 总收益率 | 总体收益率百分比 |
| 总交易次数 | 完成的交易总数 |
| 盈利交易 | 盈利的交易次数 |
| 亏损交易 | 亏损的交易次数 |
| 胜率 | 盈利交易占比 |
| 最大回撤 | 最大资金回撤百分比 |
| 夏普比率 | 风险调整后的收益指标 |

## 技术指标

系统内置以下技术指标：

- **SMA**：简单移动平均线
- **EMA**：指数移动平均线
- **RSI**：相对强弱指数
- **MACD**：移动平均收敛发散
- **布林带**：价格波动区间指标
- **ATR**：平均真实波幅
- **成交量 SMA**：成交量移动平均

## 风险管理

系统提供以下风险管理功能：

- **仓位管理**：限制单笔交易的最大仓位
- **止损机制**：当亏损达到设定比例时自动平仓
- **止盈机制**：当盈利达到设定比例时自动平仓
- **仓位计算**：根据账户余额和风险比例计算交易仓位

## 扩展开发

### 添加新策略

1. 在 `src/strategy/` 目录下创建新的策略文件
2. 实现 `Strategy` trait
3. 在 `src/strategy/mod.rs` 中导出新策略
4. 在 `main.rs` 中初始化并使用新策略

示例：

```rust
use crate::data::Kline;
use crate::strategy::{Signal, Strategy};
use anyhow::Result;

pub struct MyStrategy {
    name: String,
    // 添加策略参数
}

impl Strategy for MyStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_data(&mut self, klines: &[Kline]) -> Result<Signal> {
        // 实现策略逻辑
        Ok(Signal::Hold)
    }
}
```

### 添加交易所适配器

1. 在 `src/data/` 目录下创建新的适配器文件
2. 实现 `Exchange` trait
3. 在 `src/data/mod.rs` 中导出新适配器

## 注意事项

- 本系统仅供学习和研究使用，不构成投资建议
- 历史回测结果不代表未来收益
- 实际交易前请充分测试并了解相关风险
- 建议使用模拟交易进行实盘验证

## 许可证

本项目仅供学习研究使用。
