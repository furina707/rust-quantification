use crate::data::Kline;

pub struct TechnicalIndicators;

impl TechnicalIndicators {
    pub fn sma(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(prices.len());
        
        for i in 0..prices.len() {
            if i < period - 1 {
                result.push(None);
            } else {
                let sum: f64 = prices[i + 1 - period..=i].iter().sum();
                result.push(Some(sum / period as f64));
            }
        }
        
        result
    }

    pub fn ema(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(prices.len());
        let multiplier = 2.0 / (period as f64 + 1.0);
        
        let sma = Self::sma(prices, period);
        let mut prev_ema: Option<f64> = None;
        
        for i in 0..prices.len() {
            if i < period - 1 {
                result.push(None);
            } else if i == period - 1 {
                if let Some(sma_val) = sma[i] {
                    result.push(Some(sma_val));
                    prev_ema = Some(sma_val);
                } else {
                    result.push(None);
                }
            } else {
                if let Some(prev) = prev_ema {
                    let ema = (prices[i] - prev) * multiplier + prev;
                    result.push(Some(ema));
                    prev_ema = Some(ema);
                } else {
                    result.push(None);
                }
            }
        }
        
        result
    }

    pub fn rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(prices.len());
        
        if prices.len() < period + 1 {
            return vec![None; prices.len()];
        }
        
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }
        
        let mut avg_gain: Option<f64> = None;
        let mut avg_loss: Option<f64> = None;
        
        for i in 0..prices.len() {
            if i < period {
                result.push(None);
            } else if i == period {
                let gain_sum: f64 = gains[i - period..i].iter().sum();
                let loss_sum: f64 = losses[i - period..i].iter().sum();
                avg_gain = Some(gain_sum / period as f64);
                avg_loss = Some(loss_sum / period as f64);
                
                if let (Some(ag), Some(al)) = (avg_gain, avg_loss) {
                    if al == 0.0 {
                        result.push(Some(100.0));
                    } else {
                        let rs = ag / al;
                        let rsi = 100.0 - (100.0 / (1.0 + rs));
                        result.push(Some(rsi));
                    }
                } else {
                    result.push(None);
                }
            } else {
                if let (Some(ag), Some(al)) = (avg_gain, avg_loss) {
                    let current_gain = gains[i - 1];
                    let current_loss = losses[i - 1];
                    
                    let new_avg_gain = (ag * (period as f64 - 1.0) + current_gain) / period as f64;
                    let new_avg_loss = (al * (period as f64 - 1.0) + current_loss) / period as f64;
                    
                    avg_gain = Some(new_avg_gain);
                    avg_loss = Some(new_avg_loss);
                    
                    if new_avg_loss == 0.0 {
                        result.push(Some(100.0));
                    } else {
                        let rs = new_avg_gain / new_avg_loss;
                        let rsi = 100.0 - (100.0 / (1.0 + rs));
                        result.push(Some(rsi));
                    }
                } else {
                    result.push(None);
                }
            }
        }
        
        result
    }

    pub fn macd(
        prices: &[f64],
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        let fast_ema = Self::ema(prices, fast_period);
        let slow_ema = Self::ema(prices, slow_period);
        
        let mut macd_line = Vec::with_capacity(prices.len());
        for i in 0..prices.len() {
            if let (Some(fast), Some(slow)) = (fast_ema[i], slow_ema[i]) {
                macd_line.push(Some(fast - slow));
            } else {
                macd_line.push(None);
            }
        }
        
        let macd_values: Vec<f64> = macd_line
            .iter()
            .filter_map(|&x| x)
            .collect();
        
        let signal_ema = Self::ema(&macd_values, signal_period);
        
        let mut signal_line = Vec::with_capacity(prices.len());
        let mut signal_idx = 0;
        for i in 0..prices.len() {
            if macd_line[i].is_some() {
                if signal_idx < signal_ema.len() {
                    signal_line.push(signal_ema[signal_idx]);
                    signal_idx += 1;
                } else {
                    signal_line.push(None);
                }
            } else {
                signal_line.push(None);
            }
        }
        
        let mut histogram = Vec::with_capacity(prices.len());
        for i in 0..prices.len() {
            if let (Some(macd), Some(signal)) = (macd_line[i], signal_line[i]) {
                histogram.push(Some(macd - signal));
            } else {
                histogram.push(None);
            }
        }
        
        (macd_line, signal_line, histogram)
    }

    pub fn bollinger_bands(
        prices: &[f64],
        period: usize,
        std_dev: f64,
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        let sma = Self::sma(prices, period);
        let mut upper = Vec::with_capacity(prices.len());
        let mut lower = Vec::with_capacity(prices.len());
        
        for i in 0..prices.len() {
            if i < period - 1 {
                upper.push(None);
                lower.push(None);
            } else {
                let slice = &prices[i + 1 - period..=i];
                let mean = slice.iter().sum::<f64>() / period as f64;
                let variance = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
                let std = variance.sqrt();
                
                upper.push(Some(mean + std_dev * std));
                lower.push(Some(mean - std_dev * std));
            }
        }
        
        (upper, sma, lower)
    }

    pub fn atr(klines: &[Kline], period: usize) -> Vec<Option<f64>> {
        let mut tr_values = Vec::with_capacity(klines.len());
        
        for i in 0..klines.len() {
            if i == 0 {
                tr_values.push(klines[i].high - klines[i].low);
            } else {
                let tr1 = klines[i].high - klines[i].low;
                let tr2 = (klines[i].high - klines[i - 1].close).abs();
                let tr3 = (klines[i].low - klines[i - 1].close).abs();
                tr_values.push(tr1.max(tr2).max(tr3));
            }
        }
        
        Self::sma(&tr_values, period)
    }

    pub fn volume_sma(volumes: &[f64], period: usize) -> Vec<Option<f64>> {
        Self::sma(volumes, period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let sma = TechnicalIndicators::sma(&prices, 3);
        
        assert_eq!(sma[0], None);
        assert_eq!(sma[1], None);
        assert_eq!(sma[2], Some(2.0));
        assert_eq!(sma[3], Some(3.0));
        assert_eq!(sma[9], Some(9.0));
    }

    #[test]
    fn test_rsi() {
        let prices = vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
            45.89, 46.03, 45.61, 46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64,
        ];
        let rsi = TechnicalIndicators::rsi(&prices, 14);
        
        assert!(rsi[14].is_some());
        if let Some(rsi_val) = rsi[19] {
            assert!(rsi_val > 0.0 && rsi_val < 100.0);
        }
    }
}
