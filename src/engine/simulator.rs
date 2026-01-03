//! Core Simulation Engine
//!
//! This module orchestrates the multi-oracle simulation, detecting
//! desynchronization windows and collecting metrics.

use crate::config::SimulationConfig;
use crate::engine::metrics::MetricsCollector;
use crate::oracle::{deviation_pct, ema::EmaOracle, spot::SpotOracle, twap::TwapOracle, Oracle};
use crate::types::{
    DesyncEvent, DesyncFlag, OracleStats, OracleStatsSummary, PricePoint, RiskLevel,
    SimulationMetadata, SimulationResult,
};
use std::time::Instant;

/// Main simulation engine
pub struct Simulator {
    /// Spot oracle
    spot: SpotOracle,
    /// TWAP oracle
    twap: TwapOracle,
    /// EMA oracle
    ema: EmaOracle,
    /// Metrics collector
    metrics: MetricsCollector,
    /// Configuration
    config: SimulationConfig,
    /// Detected desync events
    events: Vec<DesyncEvent>,
    /// Active desync tracking
    active_desync: Option<ActiveDesync>,
}

/// Tracks an active desynchronization window
#[derive(Debug)]
struct ActiveDesync {
    start_timestamp: u64,
    oracle_pair: String,
    max_deviation: f64,
    flags: Vec<DesyncFlag>,
}

impl Simulator {
    /// Create a new simulator from configuration
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            spot: SpotOracle::new(),
            twap: TwapOracle::new(config.twap_window_sec),
            ema: EmaOracle::new(config.ema_alpha),
            metrics: MetricsCollector::new(),
            config,
            events: Vec::new(),
            active_desync: None,
        }
    }

    /// Run the full simulation
    pub fn run(&mut self) -> SimulationResult {
        let start_time = Instant::now();

        // Process each price point
        for price_point in self.config.prices.clone() {
            self.process_price_point(&price_point);
        }

        // Close any active desync window
        if let Some(active) = self.active_desync.take() {
            self.close_desync_window(
                active,
                self.config.prices.last().map(|p| p.timestamp).unwrap_or(0),
            );
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        self.generate_result(duration_ms)
    }

    /// Process a single price point
    fn process_price_point(&mut self, price_point: &PricePoint) {
        // Update all oracles
        self.spot.update(price_point);
        self.twap.update(price_point);
        self.ema.update(price_point);

        // Get current prices
        let spot_price = self.spot.get_price().unwrap_or(0.0);
        let twap_price = self.twap.get_price().unwrap_or(spot_price);
        let ema_price = self.ema.get_price().unwrap_or(spot_price);

        // Calculate deviations
        let spot_twap_dev = deviation_pct(spot_price, twap_price);
        let spot_ema_dev = deviation_pct(spot_price, ema_price);
        let twap_ema_dev = deviation_pct(twap_price, ema_price);

        // Update metrics
        self.metrics
            .record_deviation(spot_twap_dev, spot_ema_dev, twap_ema_dev);

        // Check for desync conditions
        let mut current_flags: Vec<DesyncFlag> = Vec::new();
        let max_deviation = spot_twap_dev.max(spot_ema_dev).max(twap_ema_dev);

        // Check staleness
        if self
            .spot
            .is_stale(price_point.timestamp, self.config.max_oracle_delay_sec)
        {
            current_flags.push(DesyncFlag::StalePriceWindow);
        }

        // Check deviation thresholds
        if spot_twap_dev > self.config.high_deviation_threshold_pct {
            current_flags.push(DesyncFlag::SpotTwapDivergence);
        }
        if spot_ema_dev > self.config.high_deviation_threshold_pct {
            current_flags.push(DesyncFlag::SpotEmaDivergence);
        }
        if twap_ema_dev > self.config.high_deviation_threshold_pct {
            current_flags.push(DesyncFlag::TwapEmaDivergence);
        }

        // Check for high volatility
        let volatility = self.spot.volatility();
        if volatility > 5.0 {
            current_flags.push(DesyncFlag::HighVolatility);
        }

        // Handle desync window tracking
        if max_deviation > 1.0 || !current_flags.is_empty() {
            // We're in a desync condition
            match &mut self.active_desync {
                Some(active) => {
                    // Update existing window
                    active.max_deviation = active.max_deviation.max(max_deviation);
                    for flag in current_flags {
                        if !active.flags.contains(&flag) {
                            active.flags.push(flag);
                        }
                    }
                }
                None => {
                    // Start new window
                    self.active_desync = Some(ActiveDesync {
                        start_timestamp: price_point.timestamp,
                        oracle_pair: "spot-twap-ema".to_string(),
                        max_deviation,
                        flags: current_flags,
                    });
                }
            }
        } else if let Some(active) = self.active_desync.take() {
            // End the desync window
            self.close_desync_window(active, price_point.timestamp);
        }

        // Update timestamp tracking
        self.metrics.update_timestamp(price_point.timestamp);
    }

    /// Close a desync window and record the event
    fn close_desync_window(&mut self, active: ActiveDesync, end_timestamp: u64) {
        let duration = end_timestamp.saturating_sub(active.start_timestamp);
        let risk_level = RiskLevel::classify(active.max_deviation, duration);

        let event = DesyncEvent {
            start_timestamp: active.start_timestamp,
            end_timestamp,
            max_deviation_pct: active.max_deviation,
            oracle_pair: active.oracle_pair,
            risk_level,
            flags: active.flags,
        };

        self.events.push(event);
    }

    /// Generate the final simulation result
    fn generate_result(&self, duration_ms: u64) -> SimulationResult {
        // Find worst event
        let (worst_deviation, worst_timestamp, worst_duration) =
            self.events
                .iter()
                .fold((0.0, 0u64, 0u64), |(max_dev, ts, dur), event| {
                    if event.max_deviation_pct > max_dev {
                        (
                            event.max_deviation_pct,
                            event.start_timestamp,
                            event.duration_sec(),
                        )
                    } else {
                        (max_dev, ts, dur)
                    }
                });

        // Collect all unique flags
        let mut all_flags: Vec<DesyncFlag> =
            self.events.iter().flat_map(|e| e.flags.clone()).collect();
        all_flags.sort_by_key(|f| format!("{:?}", f));
        all_flags.dedup();

        // Calculate overall risk level
        let overall_risk = if self
            .events
            .iter()
            .any(|e| e.risk_level == RiskLevel::Critical)
        {
            RiskLevel::Critical
        } else if self.events.iter().any(|e| e.risk_level == RiskLevel::High) {
            RiskLevel::High
        } else if self
            .events
            .iter()
            .any(|e| e.risk_level == RiskLevel::Medium)
        {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Build oracle statistics
        let oracle_stats = OracleStatsSummary {
            spot: OracleStats {
                update_count: self.spot.update_count() as u64,
                price_avg: self.spot.average_price().unwrap_or(0.0),
                price_max: self.spot.max_price().unwrap_or(0.0),
                price_min: self.spot.min_price().unwrap_or(0.0),
                price_std: self.spot.volatility(),
                avg_update_interval_sec: self.config.avg_update_interval_sec(),
                max_update_gap_sec: 0, // Would need additional tracking
            },
            twap: OracleStats {
                update_count: self.twap.observation_count() as u64,
                ..Default::default()
            },
            ema: OracleStats {
                update_count: self.ema.update_count(),
                ..Default::default()
            },
            spot_twap_deviation_avg: self.metrics.avg_spot_twap_deviation(),
            spot_twap_deviation_max: self.metrics.max_spot_twap_deviation(),
            spot_ema_deviation_avg: self.metrics.avg_spot_ema_deviation(),
            ema_lag_max_sec: 0, // Would need additional tracking
        };

        SimulationResult {
            max_desync_percent: worst_deviation,
            desync_window_sec: worst_duration,
            worst_timestamp,
            risk_level: overall_risk,
            flags: all_flags,
            events: self.events.clone(),
            oracle_stats,
            metadata: SimulationMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                run_timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                price_points_count: self.config.prices.len(),
                simulation_duration_ms: duration_ms,
                config_summary: self.config.summary(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(prices: Vec<PricePoint>) -> SimulationConfig {
        SimulationConfig {
            prices,
            twap_window_sec: 300,
            ema_alpha: 0.1,
            max_oracle_delay_sec: 600,
            liquidation_threshold: 0.85,
            high_deviation_threshold_pct: 2.0,
            high_duration_threshold_sec: 120,
            verbose: false,
            description: None,
        }
    }

    #[test]
    fn test_simulator_basic() {
        let prices = vec![
            PricePoint::new(1000, 100.0),
            PricePoint::new(1060, 101.0),
            PricePoint::new(1120, 102.0),
        ];

        let config = create_test_config(prices);
        let mut sim = Simulator::new(config);
        let result = sim.run();

        assert_eq!(result.metadata.price_points_count, 3);
        assert!(result.max_desync_percent >= 0.0);
    }

    #[test]
    fn test_simulator_with_desync() {
        let prices = vec![
            PricePoint::new(1000, 100.0),
            PricePoint::new(1060, 100.0),
            PricePoint::new(1120, 110.0), // 10% jump
            PricePoint::new(1180, 110.0),
            PricePoint::new(1240, 100.0), // Back to normal
        ];

        let config = create_test_config(prices);
        let mut sim = Simulator::new(config);
        let result = sim.run();

        // Should detect some deviation
        assert!(result.max_desync_percent > 0.0);
    }

    #[test]
    fn test_simulator_stable_prices() {
        let prices: Vec<PricePoint> = (0..10)
            .map(|i| PricePoint::new(1000 + i * 60, 100.0))
            .collect();

        let config = create_test_config(prices);
        let mut sim = Simulator::new(config);
        let result = sim.run();

        // Stable prices should result in low risk
        assert!(matches!(result.risk_level, RiskLevel::Low));
    }
}
