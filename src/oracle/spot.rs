//! Spot Price Oracle Implementation
//!
//! The spot oracle represents the current market price without any
//! smoothing or averaging. It always returns the latest price.

use super::Oracle;
use crate::types::PricePoint;

/// Spot price oracle - returns the latest price immediately
#[derive(Debug, Clone, Default)]
pub struct SpotOracle {
    /// Current price
    current_price: Option<f64>,
    /// Timestamp of last update
    last_timestamp: Option<u64>,
    /// History of prices for analysis
    price_history: Vec<PricePoint>,
    /// Maximum history size to keep
    max_history_size: usize,
}

impl SpotOracle {
    /// Create a new spot oracle
    pub fn new() -> Self {
        Self {
            current_price: None,
            last_timestamp: None,
            price_history: Vec::new(),
            max_history_size: 1000,
        }
    }

    /// Create with custom history size
    pub fn with_history_size(max_size: usize) -> Self {
        Self {
            max_history_size: max_size,
            ..Self::new()
        }
    }

    /// Get the full price history
    pub fn history(&self) -> &[PricePoint] {
        &self.price_history
    }

    /// Get price at a specific timestamp (or nearest before)
    pub fn get_price_at(&self, timestamp: u64) -> Option<f64> {
        // Find the latest price before or at the given timestamp
        self.price_history
            .iter()
            .rev()
            .find(|p| p.timestamp <= timestamp)
            .map(|p| p.price)
    }

    /// Calculate price volatility (standard deviation of returns)
    pub fn volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        // Calculate returns
        let returns: Vec<f64> = self
            .price_history
            .windows(2)
            .map(|w| {
                if w[0].price == 0.0 {
                    0.0
                } else {
                    (w[1].price - w[0].price) / w[0].price
                }
            })
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        // Calculate mean
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;

        // Calculate variance
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

        variance.sqrt() * 100.0 // Return as percentage
    }

    /// Get the update count
    pub fn update_count(&self) -> usize {
        self.price_history.len()
    }

    /// Get average price
    pub fn average_price(&self) -> Option<f64> {
        if self.price_history.is_empty() {
            return None;
        }
        Some(
            self.price_history.iter().map(|p| p.price).sum::<f64>()
                / self.price_history.len() as f64,
        )
    }

    /// Get min price
    pub fn min_price(&self) -> Option<f64> {
        self.price_history
            .iter()
            .map(|p| p.price)
            .fold(None, |min, price| {
                Some(min.map_or(price, |m: f64| m.min(price)))
            })
    }

    /// Get max price
    pub fn max_price(&self) -> Option<f64> {
        self.price_history
            .iter()
            .map(|p| p.price)
            .fold(None, |max, price| {
                Some(max.map_or(price, |m: f64| m.max(price)))
            })
    }
}

impl Oracle for SpotOracle {
    fn name(&self) -> &'static str {
        "SPOT"
    }

    fn update(&mut self, price_point: &PricePoint) {
        self.current_price = Some(price_point.price);
        self.last_timestamp = Some(price_point.timestamp);

        // Add to history
        self.price_history.push(*price_point);

        // Trim history if needed
        if self.price_history.len() > self.max_history_size {
            self.price_history.remove(0);
        }
    }

    fn get_price(&self) -> Option<f64> {
        self.current_price
    }

    fn last_update_timestamp(&self) -> Option<u64> {
        self.last_timestamp
    }

    fn reset(&mut self) {
        self.current_price = None;
        self.last_timestamp = None;
        self.price_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_oracle_basic() {
        let mut oracle = SpotOracle::new();

        assert!(oracle.get_price().is_none());

        oracle.update(&PricePoint::new(1000, 100.0));
        assert_eq!(oracle.get_price(), Some(100.0));
        assert_eq!(oracle.last_update_timestamp(), Some(1000));

        oracle.update(&PricePoint::new(1060, 105.0));
        assert_eq!(oracle.get_price(), Some(105.0));
    }

    #[test]
    fn test_spot_oracle_history() {
        let mut oracle = SpotOracle::new();

        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 105.0));
        oracle.update(&PricePoint::new(1120, 103.0));

        assert_eq!(oracle.update_count(), 3);
        assert_eq!(oracle.get_price_at(1060), Some(105.0));
        assert_eq!(oracle.get_price_at(1090), Some(105.0)); // Before 1120
    }

    #[test]
    fn test_spot_oracle_staleness() {
        let mut oracle = SpotOracle::new();
        oracle.update(&PricePoint::new(1000, 100.0));

        assert!(!oracle.is_stale(1500, 600)); // 500 < 600
        assert!(oracle.is_stale(1700, 600)); // 700 > 600
    }

    #[test]
    fn test_spot_oracle_volatility() {
        let mut oracle = SpotOracle::new();

        // Add some prices with volatility
        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 105.0)); // +5%
        oracle.update(&PricePoint::new(1120, 100.0)); // -4.76%
        oracle.update(&PricePoint::new(1180, 110.0)); // +10%

        let vol = oracle.volatility();
        assert!(vol > 0.0);
    }

    #[test]
    fn test_spot_oracle_reset() {
        let mut oracle = SpotOracle::new();
        oracle.update(&PricePoint::new(1000, 100.0));

        oracle.reset();

        assert!(oracle.get_price().is_none());
        assert!(oracle.history().is_empty());
    }
}
