//! Metrics Collection and Risk Calculation
//!
//! This module provides utilities for collecting simulation metrics
//! and calculating risk scores.

/// Metrics collector for aggregating simulation data
#[derive(Debug, Default)]
pub struct MetricsCollector {
    /// All spot-TWAP deviations recorded
    spot_twap_deviations: Vec<f64>,
    /// All spot-EMA deviations recorded
    spot_ema_deviations: Vec<f64>,
    /// All TWAP-EMA deviations recorded
    twap_ema_deviations: Vec<f64>,
    /// Timestamps recorded
    timestamps: Vec<u64>,
    /// Last recorded timestamp
    last_timestamp: Option<u64>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record deviation measurements
    pub fn record_deviation(&mut self, spot_twap: f64, spot_ema: f64, twap_ema: f64) {
        self.spot_twap_deviations.push(spot_twap);
        self.spot_ema_deviations.push(spot_ema);
        self.twap_ema_deviations.push(twap_ema);
    }

    /// Update timestamp tracking
    pub fn update_timestamp(&mut self, timestamp: u64) {
        self.timestamps.push(timestamp);
        self.last_timestamp = Some(timestamp);
    }

    /// Get average spot-TWAP deviation
    pub fn avg_spot_twap_deviation(&self) -> f64 {
        if self.spot_twap_deviations.is_empty() {
            return 0.0;
        }
        self.spot_twap_deviations.iter().sum::<f64>() / self.spot_twap_deviations.len() as f64
    }

    /// Get maximum spot-TWAP deviation
    pub fn max_spot_twap_deviation(&self) -> f64 {
        self.spot_twap_deviations
            .iter()
            .fold(0.0, |max, &dev| max.max(dev))
    }

    /// Get average spot-EMA deviation
    pub fn avg_spot_ema_deviation(&self) -> f64 {
        if self.spot_ema_deviations.is_empty() {
            return 0.0;
        }
        self.spot_ema_deviations.iter().sum::<f64>() / self.spot_ema_deviations.len() as f64
    }

    /// Get maximum spot-EMA deviation
    pub fn max_spot_ema_deviation(&self) -> f64 {
        self.spot_ema_deviations
            .iter()
            .fold(0.0, |max, &dev| max.max(dev))
    }

    /// Get average TWAP-EMA deviation
    pub fn avg_twap_ema_deviation(&self) -> f64 {
        if self.twap_ema_deviations.is_empty() {
            return 0.0;
        }
        self.twap_ema_deviations.iter().sum::<f64>() / self.twap_ema_deviations.len() as f64
    }

    /// Get maximum TWAP-EMA deviation
    pub fn max_twap_ema_deviation(&self) -> f64 {
        self.twap_ema_deviations
            .iter()
            .fold(0.0, |max, &dev| max.max(dev))
    }

    /// Get overall maximum deviation across all oracle pairs
    pub fn max_overall_deviation(&self) -> f64 {
        self.max_spot_twap_deviation()
            .max(self.max_spot_ema_deviation())
            .max(self.max_twap_ema_deviation())
    }

    /// Get total number of samples
    pub fn sample_count(&self) -> usize {
        self.spot_twap_deviations.len()
    }

    /// Calculate standard deviation of spot-TWAP deviations
    pub fn std_spot_twap_deviation(&self) -> f64 {
        calculate_std(&self.spot_twap_deviations)
    }

    /// Calculate standard deviation of spot-EMA deviations
    pub fn std_spot_ema_deviation(&self) -> f64 {
        calculate_std(&self.spot_ema_deviations)
    }

    /// Get percentile of spot-TWAP deviation
    pub fn percentile_spot_twap(&self, p: f64) -> f64 {
        calculate_percentile(&self.spot_twap_deviations, p)
    }

    /// Calculate time span covered
    pub fn time_span_sec(&self) -> u64 {
        if self.timestamps.len() < 2 {
            return 0;
        }
        self.timestamps.last().unwrap_or(&0) - self.timestamps.first().unwrap_or(&0)
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.spot_twap_deviations.clear();
        self.spot_ema_deviations.clear();
        self.twap_ema_deviations.clear();
        self.timestamps.clear();
        self.last_timestamp = None;
    }
}

/// Calculate standard deviation of a slice
fn calculate_std(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

/// Calculate percentile of a slice (0-100)
fn calculate_percentile(data: &[f64], percentile: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut sorted: Vec<f64> = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = (percentile / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

/// Volatility-adjusted deviation calculation
pub fn volatility_adjusted_deviation(deviation: f64, volatility: f64) -> f64 {
    if volatility <= 0.0 {
        return deviation;
    }
    deviation / volatility
}

/// Calculate Z-score for deviation
pub fn deviation_zscore(deviation: f64, mean: f64, std: f64) -> f64 {
    if std <= 0.0 {
        return 0.0;
    }
    (deviation - mean) / std
}

/// Time-to-recover estimation
pub fn estimate_time_to_recover(
    current_deviation: f64,
    target_deviation: f64,
    ema_alpha: f64,
    update_interval_sec: f64,
) -> f64 {
    if current_deviation <= target_deviation || ema_alpha <= 0.0 || ema_alpha >= 1.0 {
        return 0.0;
    }

    // Number of updates needed for EMA to converge
    // deviation_n = deviation_0 * (1-alpha)^n
    // n = ln(target/current) / ln(1-alpha)
    let n = (target_deviation / current_deviation).ln() / (1.0 - ema_alpha).ln();
    n * update_interval_sec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_basic() {
        let mut collector = MetricsCollector::new();

        collector.record_deviation(1.0, 0.5, 0.5);
        collector.record_deviation(2.0, 1.0, 1.0);
        collector.record_deviation(3.0, 1.5, 1.5);

        assert_eq!(collector.sample_count(), 3);
        assert!((collector.avg_spot_twap_deviation() - 2.0).abs() < 0.001);
        assert!((collector.max_spot_twap_deviation() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_std() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std = calculate_std(&data);
        // Known std for this data is 2.0
        assert!((std - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        assert!((calculate_percentile(&data, 0.0) - 1.0).abs() < 0.1);
        assert!((calculate_percentile(&data, 50.0) - 5.5).abs() < 1.0); // Median of 1-10 is ~5.5
        assert!((calculate_percentile(&data, 100.0) - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_volatility_adjusted() {
        let dev = volatility_adjusted_deviation(5.0, 2.5);
        assert!((dev - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_time_to_recover() {
        let time = estimate_time_to_recover(10.0, 1.0, 0.1, 60.0);
        // With alpha=0.1, need ~22 updates to go from 10% to 1%
        // 22 * 60 = 1320 seconds
        assert!(time > 1000.0 && time < 2000.0);
    }
}
