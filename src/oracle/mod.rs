//! Oracle implementations module
//!
//! This module provides different oracle type implementations:
//! - Spot: Instant price oracle
//! - TWAP: Time-Weighted Average Price
//! - EMA: Exponential Moving Average

pub mod ema;
pub mod spot;
pub mod twap;

use crate::types::PricePoint;

/// Common trait for all oracle types
pub trait Oracle {
    /// Get the oracle type name
    fn name(&self) -> &'static str;

    /// Update the oracle with a new price point
    fn update(&mut self, price_point: &PricePoint);

    /// Get the current oracle price
    fn get_price(&self) -> Option<f64>;

    /// Get the last update timestamp
    fn last_update_timestamp(&self) -> Option<u64>;

    /// Check if the oracle is stale (no update for too long)
    fn is_stale(&self, current_timestamp: u64, max_delay_sec: u64) -> bool {
        match self.last_update_timestamp() {
            Some(last) => current_timestamp.saturating_sub(last) > max_delay_sec,
            None => true, // No updates yet = stale
        }
    }

    /// Reset the oracle state
    fn reset(&mut self);
}

/// Calculate percentage deviation between two prices
pub fn deviation_pct(price_a: f64, price_b: f64) -> f64 {
    if price_b == 0.0 {
        return 0.0;
    }
    ((price_a - price_b) / price_b).abs() * 100.0
}

/// Calculate log-return deviation between two prices
pub fn log_deviation(price_a: f64, price_b: f64) -> f64 {
    if price_a <= 0.0 || price_b <= 0.0 {
        return 0.0;
    }
    (price_a / price_b).ln().abs() * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deviation_pct() {
        assert!((deviation_pct(102.0, 100.0) - 2.0).abs() < 0.001);
        assert!((deviation_pct(98.0, 100.0) - 2.0).abs() < 0.001);
        assert_eq!(deviation_pct(100.0, 0.0), 0.0);
    }

    #[test]
    fn test_log_deviation() {
        // ln(1.02) ≈ 0.0198 → 1.98%
        let dev = log_deviation(102.0, 100.0);
        assert!(dev > 1.9 && dev < 2.1);
    }
}
