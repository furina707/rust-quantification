use crate::data::Side;

pub struct RiskManager {
    max_position_size: f64,
    stop_loss_pct: f64,
    take_profit_pct: f64,
}

impl RiskManager {
    pub fn new(max_position_size: f64, stop_loss_pct: f64, take_profit_pct: f64) -> Self {
        Self {
            max_position_size,
            stop_loss_pct,
            take_profit_pct,
        }
    }

    pub fn check_position_size(&self, quantity: f64, price: f64) -> bool {
        let position_value = quantity * price;
        position_value <= self.max_position_size
    }

    pub fn check_stop_loss(&self, entry_price: f64, current_price: f64, side: Side) -> bool {
        match side {
            Side::Buy => {
                let loss_pct = (entry_price - current_price) / entry_price * 100.0;
                loss_pct >= self.stop_loss_pct
            }
            Side::Sell => {
                let loss_pct = (current_price - entry_price) / entry_price * 100.0;
                loss_pct >= self.stop_loss_pct
            }
        }
    }

    pub fn check_take_profit(&self, entry_price: f64, current_price: f64, side: Side) -> bool {
        match side {
            Side::Buy => {
                let profit_pct = (current_price - entry_price) / entry_price * 100.0;
                profit_pct >= self.take_profit_pct
            }
            Side::Sell => {
                let profit_pct = (entry_price - current_price) / entry_price * 100.0;
                profit_pct >= self.take_profit_pct
            }
        }
    }

    pub fn calculate_position_size(&self, balance: f64, risk_pct: f64, entry_price: f64, stop_loss_price: f64) -> f64 {
        let risk_amount = balance * risk_pct;
        let price_risk = (entry_price - stop_loss_price).abs();
        risk_amount / price_risk
    }

    pub fn calculate_stop_loss(&self, entry_price: f64, side: Side) -> f64 {
        match side {
            Side::Buy => entry_price * (1.0 - self.stop_loss_pct / 100.0),
            Side::Sell => entry_price * (1.0 + self.stop_loss_pct / 100.0),
        }
    }

    pub fn calculate_take_profit(&self, entry_price: f64, side: Side) -> f64 {
        match side {
            Side::Buy => entry_price * (1.0 + self.take_profit_pct / 100.0),
            Side::Sell => entry_price * (1.0 - self.take_profit_pct / 100.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_loss() {
        let risk_manager = RiskManager::new(1000.0, 2.0, 5.0);
        
        let stop_loss = risk_manager.calculate_stop_loss(100.0, Side::Buy);
        assert!((stop_loss - 98.0).abs() < 0.01);
    }

    #[test]
    fn test_take_profit() {
        let risk_manager = RiskManager::new(1000.0, 2.0, 5.0);
        
        let take_profit = risk_manager.calculate_take_profit(100.0, Side::Buy);
        assert!((take_profit - 105.0).abs() < 0.01);
    }
}
