//! TWAP (Time-Weighted Average Price) Oracle Implementation
//!
//! TWAP calculates the average price over a specified time window,
//! weighted by the time between each price update.

use super::Oracle;
use crate::types::PricePoint;
use std::collections::VecDeque;

/// Time-Weighted Average Price Oracle
#[derive(Debug, Clone)]
pub struct TwapOracle {
    /// Window size in seconds
    window_sec: u64,
    /// Price observations within the window
    observations: VecDeque<PricePoint>,
    /// Current TWAP value
    current_twap: Option<f64>,
    /// Last calculation timestamp
    last_timestamp: Option<u64>,
}

impl TwapOracle {
    /// Create a new TWAP oracle with specified window size
    pub fn new(window_sec: u64) -> Self {
        Self {
            window_sec,
            observations: VecDeque::new(),
            current_twap: None,
            last_timestamp: None,
        }
    }

    /// Get the window size
    pub fn window_sec(&self) -> u64 {
        self.window_sec
    }

    /// Get number of observations in current window
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Calculate TWAP from current observations
    fn calculate_twap(&self) -> Option<f64> {
        if self.observations.len() < 2 {
            return self.observations.front().map(|p| p.price);
        }

        let mut weighted_sum = 0.0;
        let mut total_time = 0u64;

        // Calculate time-weighted sum
        for window in self.observations.iter().collect::<Vec<_>>().windows(2) {
            let time_delta = window[1].timestamp - window[0].timestamp;
            // Weight by the price during this time period (use average of start/end)
            let avg_price = (window[0].price + window[1].price) / 2.0;
            weighted_sum += avg_price * time_delta as f64;
            total_time += time_delta;
        }

        if total_time == 0 {
            return self.observations.back().map(|p| p.price);
        }

        Some(weighted_sum / total_time as f64)
    }

    /// Prune observations outside the window
    fn prune_old_observations(&mut self, current_timestamp: u64) {
        let window_start = current_timestamp.saturating_sub(self.window_sec);

        while let Some(front) = self.observations.front() {
            if front.timestamp < window_start {
                self.observations.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get the oldest timestamp in the window
    pub fn oldest_timestamp(&self) -> Option<u64> {
        self.observations.front().map(|p| p.timestamp)
    }

    /// Get actual window coverage (time from oldest to newest observation)
    pub fn actual_window_coverage(&self) -> u64 {
        if self.observations.len() < 2 {
            return 0;
        }

        let oldest = self.observations.front().unwrap().timestamp;
        let newest = self.observations.back().unwrap().timestamp;
        newest - oldest
    }

    /// Check if the TWAP window is fully populated
    pub fn is_window_full(&self) -> bool {
        self.actual_window_coverage() >= self.window_sec
    }
}

impl Oracle for TwapOracle {
    fn name(&self) -> &'static str {
        "TWAP"
    }

    fn update(&mut self, price_point: &PricePoint) {
        // Add new observation
        self.observations.push_back(*price_point);
        self.last_timestamp = Some(price_point.timestamp);

        // Prune old observations
        self.prune_old_observations(price_point.timestamp);

        // Recalculate TWAP
        self.current_twap = self.calculate_twap();
    }

    fn get_price(&self) -> Option<f64> {
        self.current_twap
    }

    fn last_update_timestamp(&self) -> Option<u64> {
        self.last_timestamp
    }

    fn reset(&mut self) {
        self.observations.clear();
        self.current_twap = None;
        self.last_timestamp = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twap_basic() {
        let mut oracle = TwapOracle::new(300); // 5 minute window

        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 100.0));
        oracle.update(&PricePoint::new(1120, 100.0));

        // All same price, TWAP should be 100
        let twap = oracle.get_price().unwrap();
        assert!((twap - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_twap_weighted() {
        let mut oracle = TwapOracle::new(300);

        // Price at 100 for 60 seconds, then 200 for 60 seconds
        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 200.0));
        oracle.update(&PricePoint::new(1120, 200.0));

        // TWAP should be weighted average
        // (100+200)/2 * 60 + (200+200)/2 * 60) / 120 = (9000 + 12000) / 120 = 175
        let twap = oracle.get_price().unwrap();
        assert!((twap - 175.0).abs() < 1.0);
    }

    #[test]
    fn test_twap_window_pruning() {
        let mut oracle = TwapOracle::new(100); // 100 second window

        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1050, 110.0));
        oracle.update(&PricePoint::new(1150, 120.0)); // First point should be pruned

        assert_eq!(oracle.observation_count(), 2);
        assert_eq!(oracle.oldest_timestamp(), Some(1050));
    }

    #[test]
    fn test_twap_single_observation() {
        let mut oracle = TwapOracle::new(300);

        oracle.update(&PricePoint::new(1000, 100.0));

        // With single observation, should return that price
        assert_eq!(oracle.get_price(), Some(100.0));
    }

    #[test]
    fn test_twap_staleness() {
        let mut oracle = TwapOracle::new(300);
        oracle.update(&PricePoint::new(1000, 100.0));

        assert!(!oracle.is_stale(1500, 600));
        assert!(oracle.is_stale(1700, 600));
    }

    #[test]
    fn test_twap_reset() {
        let mut oracle = TwapOracle::new(300);
        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 110.0));

        oracle.reset();

        assert!(oracle.get_price().is_none());
        assert_eq!(oracle.observation_count(), 0);
    }
}
