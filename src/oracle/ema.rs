//! EMA (Exponential Moving Average) Oracle Implementation
//!
//! EMA applies exponential smoothing to price data, with more recent
//! prices having higher weight. This provides smoother price feeds
//! that are more resistant to short-term manipulation.

use super::Oracle;
use crate::types::PricePoint;

/// Exponential Moving Average Oracle
#[derive(Debug, Clone)]
pub struct EmaOracle {
    /// Smoothing factor (alpha), between 0 and 1
    /// Higher values = more weight on recent prices
    alpha: f64,
    /// Current EMA value
    current_ema: Option<f64>,
    /// Last update timestamp
    last_timestamp: Option<u64>,
    /// Number of updates received
    update_count: u64,
    /// Previous EMA (for calculating lag)
    previous_ema: Option<f64>,
}

impl EmaOracle {
    /// Create a new EMA oracle with specified alpha
    ///
    /// Alpha should be between 0 and 1:
    /// - 0.1 = slow EMA (more smoothing)
    /// - 0.3 = medium EMA
    /// - 0.5 = fast EMA (less smoothing)
    pub fn new(alpha: f64) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "Alpha must be in range (0, 1]");

        Self {
            alpha,
            current_ema: None,
            last_timestamp: None,
            update_count: 0,
            previous_ema: None,
        }
    }

    /// Create an EMA oracle with span (similar to pandas ewm)
    ///
    /// alpha = 2 / (span + 1)
    /// e.g., span=9 gives alpha ≈ 0.2
    pub fn from_span(span: u64) -> Self {
        let alpha = 2.0 / (span as f64 + 1.0);
        Self::new(alpha)
    }

    /// Get the alpha value
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Get the equivalent span
    pub fn span(&self) -> f64 {
        (2.0 / self.alpha) - 1.0
    }

    /// Get update count
    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    /// Calculate lag (difference from spot price)
    pub fn lag_from_spot(&self, spot_price: f64) -> Option<f64> {
        self.current_ema.map(|ema| (ema - spot_price).abs())
    }

    /// Calculate lag percentage from spot price
    pub fn lag_pct_from_spot(&self, spot_price: f64) -> Option<f64> {
        if spot_price == 0.0 {
            return Some(0.0);
        }
        self.current_ema
            .map(|ema| ((ema - spot_price) / spot_price).abs() * 100.0)
    }

    /// Get the previous EMA value (before last update)
    pub fn previous_ema(&self) -> Option<f64> {
        self.previous_ema
    }

    /// Calculate how many updates until EMA converges to a target price
    /// (within threshold percentage)
    pub fn updates_to_converge(&self, target_price: f64, threshold_pct: f64) -> Option<u64> {
        let current = self.current_ema?;

        if target_price == 0.0 {
            return None;
        }

        let current_deviation = ((current - target_price) / target_price).abs() * 100.0;
        if current_deviation <= threshold_pct {
            return Some(0);
        }

        // EMA convergence: after n updates, deviation = deviation_0 * (1 - alpha)^n
        // We want: deviation_0 * (1 - alpha)^n <= threshold_pct
        // n >= ln(threshold_pct / deviation_0) / ln(1 - alpha)
        let n = (threshold_pct / current_deviation).ln() / (1.0 - self.alpha).ln();
        Some(n.ceil() as u64)
    }
}

impl Oracle for EmaOracle {
    fn name(&self) -> &'static str {
        "EMA"
    }

    fn update(&mut self, price_point: &PricePoint) {
        self.previous_ema = self.current_ema;

        // EMA formula: EMA_new = alpha * price + (1 - alpha) * EMA_old
        self.current_ema = match self.current_ema {
            Some(prev_ema) => Some(self.alpha * price_point.price + (1.0 - self.alpha) * prev_ema),
            None => Some(price_point.price), // Initialize with first price
        };

        self.last_timestamp = Some(price_point.timestamp);
        self.update_count += 1;
    }

    fn get_price(&self) -> Option<f64> {
        self.current_ema
    }

    fn last_update_timestamp(&self) -> Option<u64> {
        self.last_timestamp
    }

    fn reset(&mut self) {
        self.current_ema = None;
        self.last_timestamp = None;
        self.update_count = 0;
        self.previous_ema = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_basic() {
        let mut oracle = EmaOracle::new(0.5);

        // First price becomes the EMA
        oracle.update(&PricePoint::new(1000, 100.0));
        assert_eq!(oracle.get_price(), Some(100.0));

        // Second price: EMA = 0.5 * 110 + 0.5 * 100 = 105
        oracle.update(&PricePoint::new(1060, 110.0));
        assert_eq!(oracle.get_price(), Some(105.0));

        // Third price: EMA = 0.5 * 100 + 0.5 * 105 = 102.5
        oracle.update(&PricePoint::new(1120, 100.0));
        assert_eq!(oracle.get_price(), Some(102.5));
    }

    #[test]
    fn test_ema_slow() {
        let mut oracle = EmaOracle::new(0.1); // Slow EMA

        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 200.0));

        // EMA = 0.1 * 200 + 0.9 * 100 = 110
        assert_eq!(oracle.get_price(), Some(110.0));
    }

    #[test]
    fn test_ema_from_span() {
        let oracle = EmaOracle::from_span(9);
        // alpha = 2 / (9 + 1) = 0.2
        assert!((oracle.alpha() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_ema_lag() {
        let mut oracle = EmaOracle::new(0.1);

        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 200.0));

        // EMA is 110, spot is 200
        let lag_pct = oracle.lag_pct_from_spot(200.0).unwrap();
        // |110 - 200| / 200 * 100 = 45%
        assert!((lag_pct - 45.0).abs() < 0.1);
    }

    #[test]
    fn test_ema_convergence() {
        let mut oracle = EmaOracle::new(0.1);

        oracle.update(&PricePoint::new(1000, 100.0));

        // How many updates to get within 1% of 200?
        let n = oracle.updates_to_converge(200.0, 1.0);
        assert!(n.is_some());
        assert!(n.unwrap() > 0);
    }

    #[test]
    fn test_ema_staleness() {
        let mut oracle = EmaOracle::new(0.1);
        oracle.update(&PricePoint::new(1000, 100.0));

        assert!(!oracle.is_stale(1500, 600));
        assert!(oracle.is_stale(1700, 600));
    }

    #[test]
    fn test_ema_reset() {
        let mut oracle = EmaOracle::new(0.1);
        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 110.0));

        oracle.reset();

        assert!(oracle.get_price().is_none());
        assert_eq!(oracle.update_count(), 0);
    }

    #[test]
    fn test_ema_update_count() {
        let mut oracle = EmaOracle::new(0.1);

        oracle.update(&PricePoint::new(1000, 100.0));
        oracle.update(&PricePoint::new(1060, 110.0));
        oracle.update(&PricePoint::new(1120, 120.0));

        assert_eq!(oracle.update_count(), 3);
    }
}
