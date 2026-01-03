//! Configuration parsing for OracleX
//!
//! This module handles loading and validating simulation configurations
//! from JSON files.

use crate::types::PricePoint;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main configuration structure for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Price feed data points
    pub prices: Vec<PricePoint>,

    /// TWAP calculation window in seconds
    #[serde(default = "default_twap_window")]
    pub twap_window_sec: u64,

    /// EMA smoothing factor (alpha), between 0 and 1
    /// Higher values = more weight on recent prices
    #[serde(default = "default_ema_alpha")]
    pub ema_alpha: f64,

    /// Maximum allowed oracle delay in seconds before flagging stale
    #[serde(default = "default_max_delay")]
    pub max_oracle_delay_sec: u64,

    /// Liquidation threshold (e.g., 0.85 = 85% health factor)
    #[serde(default = "default_liquidation_threshold")]
    pub liquidation_threshold: f64,

    /// Deviation threshold for HIGH risk classification (percentage)
    #[serde(default = "default_high_deviation_threshold")]
    pub high_deviation_threshold_pct: f64,

    /// Duration threshold for HIGH risk classification (seconds)
    #[serde(default = "default_high_duration_threshold")]
    pub high_duration_threshold_sec: u64,

    /// Enable verbose output
    #[serde(default)]
    pub verbose: bool,

    /// Optional description for this simulation
    #[serde(default)]
    pub description: Option<String>,
}

// Default value functions for serde
fn default_twap_window() -> u64 {
    300 // 5 minutes
}

fn default_ema_alpha() -> f64 {
    0.1 // Standard EMA smoothing
}

fn default_max_delay() -> u64 {
    600 // 10 minutes
}

fn default_liquidation_threshold() -> f64 {
    0.85 // 85% health factor
}

fn default_high_deviation_threshold() -> f64 {
    2.0 // 2% deviation
}

fn default_high_duration_threshold() -> u64 {
    120 // 2 minutes
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            prices: Vec::new(),
            twap_window_sec: default_twap_window(),
            ema_alpha: default_ema_alpha(),
            max_oracle_delay_sec: default_max_delay(),
            liquidation_threshold: default_liquidation_threshold(),
            high_deviation_threshold_pct: default_high_deviation_threshold(),
            high_duration_threshold_sec: default_high_duration_threshold(),
            verbose: false,
            description: None,
        }
    }
}

impl SimulationConfig {
    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        Self::from_json(&contents)
    }

    /// Parse configuration from a JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        let config: SimulationConfig =
            serde_json::from_str(json).context("Failed to parse JSON configuration")?;

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Check price data
        if self.prices.is_empty() {
            anyhow::bail!("Price data cannot be empty");
        }

        // Check for at least 2 price points for meaningful analysis
        if self.prices.len() < 2 {
            anyhow::bail!("Need at least 2 price points for analysis");
        }

        // Validate EMA alpha
        if self.ema_alpha <= 0.0 || self.ema_alpha > 1.0 {
            anyhow::bail!("EMA alpha must be in range (0, 1], got: {}", self.ema_alpha);
        }

        // Validate liquidation threshold
        if self.liquidation_threshold <= 0.0 || self.liquidation_threshold >= 1.0 {
            anyhow::bail!(
                "Liquidation threshold must be in range (0, 1), got: {}",
                self.liquidation_threshold
            );
        }

        // Validate TWAP window
        if self.twap_window_sec == 0 {
            anyhow::bail!("TWAP window must be greater than 0");
        }

        // Check timestamps are monotonically increasing
        for window in self.prices.windows(2) {
            if window[1].timestamp <= window[0].timestamp {
                anyhow::bail!(
                    "Timestamps must be strictly increasing. Found {} <= {}",
                    window[1].timestamp,
                    window[0].timestamp
                );
            }
        }

        // Check for valid prices (no negatives or NaN)
        for point in &self.prices {
            if point.price < 0.0 || point.price.is_nan() || point.price.is_infinite() {
                anyhow::bail!(
                    "Invalid price value at timestamp {}: {}",
                    point.timestamp,
                    point.price
                );
            }
        }

        Ok(())
    }

    /// Get time span of the price data in seconds
    pub fn time_span_sec(&self) -> u64 {
        if self.prices.len() < 2 {
            return 0;
        }
        self.prices.last().unwrap().timestamp - self.prices.first().unwrap().timestamp
    }

    /// Get average update interval in seconds
    pub fn avg_update_interval_sec(&self) -> f64 {
        if self.prices.len() < 2 {
            return 0.0;
        }
        self.time_span_sec() as f64 / (self.prices.len() - 1) as f64
    }

    /// Generate a summary string for the configuration
    pub fn summary(&self) -> String {
        format!(
            "prices={}, span={}s, twap={}s, ema_alpha={}, threshold={}",
            self.prices.len(),
            self.time_span_sec(),
            self.twap_window_sec,
            self.ema_alpha,
            self.liquidation_threshold
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prices() -> Vec<PricePoint> {
        vec![
            PricePoint::new(1700000000, 100.0),
            PricePoint::new(1700000060, 101.0),
            PricePoint::new(1700000120, 102.0),
        ]
    }

    #[test]
    fn test_config_from_json() {
        let json = r#"{
            "prices": [
                {"timestamp": 1700000000, "price": 100.0},
                {"timestamp": 1700000060, "price": 101.0}
            ],
            "twap_window_sec": 300,
            "ema_alpha": 0.1
        }"#;

        let config = SimulationConfig::from_json(json).unwrap();
        assert_eq!(config.prices.len(), 2);
        assert_eq!(config.twap_window_sec, 300);
        assert_eq!(config.ema_alpha, 0.1);
    }

    #[test]
    fn test_config_defaults() {
        let json = r#"{
            "prices": [
                {"timestamp": 1000, "price": 100.0},
                {"timestamp": 2000, "price": 101.0}
            ]
        }"#;

        let config = SimulationConfig::from_json(json).unwrap();
        assert_eq!(config.twap_window_sec, 300);
        assert_eq!(config.ema_alpha, 0.1);
        assert_eq!(config.max_oracle_delay_sec, 600);
    }

    #[test]
    fn test_validation_empty_prices() {
        let config = SimulationConfig {
            prices: vec![],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_alpha() {
        let config = SimulationConfig {
            prices: sample_prices(),
            ema_alpha: 1.5, // Invalid
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_non_increasing_timestamps() {
        let config = SimulationConfig {
            prices: vec![
                PricePoint::new(1000, 100.0),
                PricePoint::new(900, 101.0), // Decreasing!
            ],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_time_span() {
        let config = SimulationConfig {
            prices: sample_prices(),
            ..Default::default()
        };
        assert_eq!(config.time_span_sec(), 120); // 1700000120 - 1700000000
    }
}
