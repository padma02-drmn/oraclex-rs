//! Core types for OracleX
//!
//! This module defines the fundamental data structures used throughout
//! the oracle desynchronization risk simulator.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A single price observation with timestamp
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PricePoint {
    /// Unix timestamp in seconds
    pub timestamp: u64,
    /// Price value (e.g., USD per token)
    pub price: f64,
}

impl PricePoint {
    /// Create a new price point
    pub fn new(timestamp: u64, price: f64) -> Self {
        Self { timestamp, price }
    }

    /// Calculate time difference from another price point
    pub fn time_diff(&self, other: &PricePoint) -> i64 {
        self.timestamp as i64 - other.timestamp as i64
    }

    /// Calculate percentage price change from another price point
    pub fn price_change_pct(&self, other: &PricePoint) -> f64 {
        if other.price == 0.0 {
            return 0.0;
        }
        ((self.price - other.price) / other.price) * 100.0
    }
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    /// Minimal risk, normal operation
    Low,
    /// Elevated risk, monitoring recommended
    Medium,
    /// Significant risk, action may be required
    High,
    /// Severe risk, immediate attention needed
    Critical,
}

impl RiskLevel {
    /// Classify risk based on deviation percentage and duration
    pub fn classify(deviation_pct: f64, duration_sec: u64) -> Self {
        match (deviation_pct.abs(), duration_sec) {
            (d, t) if d > 5.0 || t > 600 => RiskLevel::Critical,
            (d, t) if d > 2.0 && t > 120 => RiskLevel::High,
            (d, _) if d > 1.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Get color code for terminal output
    pub fn color_code(&self) -> &'static str {
        match self {
            RiskLevel::Low => "green",
            RiskLevel::Medium => "yellow",
            RiskLevel::High => "red",
            RiskLevel::Critical => "bright red",
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Flags indicating specific risk conditions detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DesyncFlag {
    /// Price is stale (no update for too long)
    StalePriceWindow,
    /// False liquidation could occur due to oracle desync
    FalseLiquidationRisk,
    /// Escaped liquidation (should have been liquidated but wasn't)
    EscapedLiquidation,
    /// Significant deviation between spot and TWAP
    SpotTwapDivergence,
    /// Significant deviation between spot and EMA
    SpotEmaDivergence,
    /// Significant deviation between TWAP and EMA
    TwapEmaDivergence,
    /// Exceeded maximum allowed oracle delay
    MaxDelayBreached,
    /// Liquidation threshold crossed
    ThresholdCrossed,
    /// High volatility detected
    HighVolatility,
    /// Oracle update frequency mismatch
    UpdateFrequencyMismatch,
}

impl fmt::Display for DesyncFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesyncFlag::StalePriceWindow => write!(f, "STALE_PRICE_WINDOW"),
            DesyncFlag::FalseLiquidationRisk => write!(f, "FALSE_LIQUIDATION_RISK"),
            DesyncFlag::EscapedLiquidation => write!(f, "ESCAPED_LIQUIDATION"),
            DesyncFlag::SpotTwapDivergence => write!(f, "SPOT_TWAP_DIVERGENCE"),
            DesyncFlag::SpotEmaDivergence => write!(f, "SPOT_EMA_DIVERGENCE"),
            DesyncFlag::TwapEmaDivergence => write!(f, "TWAP_EMA_DIVERGENCE"),
            DesyncFlag::MaxDelayBreached => write!(f, "MAX_DELAY_BREACHED"),
            DesyncFlag::ThresholdCrossed => write!(f, "THRESHOLD_CROSSED"),
            DesyncFlag::HighVolatility => write!(f, "HIGH_VOLATILITY"),
            DesyncFlag::UpdateFrequencyMismatch => write!(f, "UPDATE_FREQUENCY_MISMATCH"),
        }
    }
}

/// Statistics for a single oracle type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleStats {
    /// Number of price updates
    pub update_count: u64,
    /// Average price value
    pub price_avg: f64,
    /// Maximum price value
    pub price_max: f64,
    /// Minimum price value
    pub price_min: f64,
    /// Price standard deviation
    pub price_std: f64,
    /// Average update interval in seconds
    pub avg_update_interval_sec: f64,
    /// Maximum update gap in seconds
    pub max_update_gap_sec: u64,
}

/// Desync event recorded during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesyncEvent {
    /// Start timestamp of the desync window
    pub start_timestamp: u64,
    /// End timestamp of the desync window
    pub end_timestamp: u64,
    /// Maximum deviation percentage during this window
    pub max_deviation_pct: f64,
    /// Oracle types involved (e.g., "spot-twap")
    pub oracle_pair: String,
    /// Risk level for this event
    pub risk_level: RiskLevel,
    /// Flags triggered by this event
    pub flags: Vec<DesyncFlag>,
}

impl DesyncEvent {
    /// Calculate duration in seconds
    pub fn duration_sec(&self) -> u64 {
        self.end_timestamp.saturating_sub(self.start_timestamp)
    }
}

/// Complete simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Maximum desync percentage observed
    pub max_desync_percent: f64,
    /// Duration of longest desync window in seconds
    pub desync_window_sec: u64,
    /// Timestamp of worst desync
    pub worst_timestamp: u64,
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// All risk flags detected
    pub flags: Vec<DesyncFlag>,
    /// Individual desync events
    pub events: Vec<DesyncEvent>,
    /// Statistics per oracle type
    pub oracle_stats: OracleStatsSummary,
    /// Simulation metadata
    pub metadata: SimulationMetadata,
}

/// Summary of oracle statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleStatsSummary {
    /// Spot oracle statistics
    pub spot: OracleStats,
    /// TWAP oracle statistics
    pub twap: OracleStats,
    /// EMA oracle statistics
    pub ema: OracleStats,
    /// Average spot-TWAP deviation
    pub spot_twap_deviation_avg: f64,
    /// Maximum spot-TWAP deviation
    pub spot_twap_deviation_max: f64,
    /// Average spot-EMA deviation
    pub spot_ema_deviation_avg: f64,
    /// Maximum EMA lag in seconds
    pub ema_lag_max_sec: u64,
}

/// Simulation run metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// Tool version
    pub version: String,
    /// Simulation run timestamp
    pub run_timestamp: u64,
    /// Number of price points processed
    pub price_points_count: usize,
    /// Simulation duration (wall time) in milliseconds
    pub simulation_duration_ms: u64,
    /// Configuration used
    pub config_summary: String,
}

impl Default for SimulationMetadata {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            run_timestamp: 0,
            price_points_count: 0,
            simulation_duration_ms: 0,
            config_summary: String::new(),
        }
    }
}

/// Liquidation state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidationState {
    /// Position is safe
    Safe,
    /// Position is at margin call level
    MarginCall,
    /// Position should be liquidated
    Liquidatable,
    /// Position was liquidated
    Liquidated,
}

/// Margin position for liquidation simulation
#[derive(Debug, Clone)]
pub struct MarginPosition {
    /// Collateral value
    pub collateral: f64,
    /// Debt value
    pub debt: f64,
    /// Liquidation threshold (e.g., 0.85 = 85%)
    pub liquidation_threshold: f64,
}

impl MarginPosition {
    /// Create a new margin position
    pub fn new(collateral: f64, debt: f64, liquidation_threshold: f64) -> Self {
        Self {
            collateral,
            debt,
            liquidation_threshold,
        }
    }

    /// Calculate health factor (collateral / debt)
    pub fn health_factor(&self) -> f64 {
        if self.debt == 0.0 {
            return f64::MAX;
        }
        self.collateral / self.debt
    }

    /// Get current liquidation state
    pub fn state(&self) -> LiquidationState {
        let hf = self.health_factor();
        if hf >= 1.0 {
            LiquidationState::Safe
        } else if hf >= self.liquidation_threshold {
            LiquidationState::MarginCall
        } else {
            LiquidationState::Liquidatable
        }
    }

    /// Update collateral based on price change
    pub fn update_collateral(&mut self, price_change_pct: f64) {
        self.collateral *= 1.0 + (price_change_pct / 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_point_change() {
        let p1 = PricePoint::new(1000, 100.0);
        let p2 = PricePoint::new(1060, 105.0);

        assert_eq!(p2.price_change_pct(&p1), 5.0);
        assert_eq!(p2.time_diff(&p1), 60);
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(RiskLevel::classify(0.5, 60), RiskLevel::Low);
        assert_eq!(RiskLevel::classify(1.5, 60), RiskLevel::Medium);
        assert_eq!(RiskLevel::classify(2.5, 180), RiskLevel::High);
        assert_eq!(RiskLevel::classify(6.0, 60), RiskLevel::Critical);
        assert_eq!(RiskLevel::classify(1.0, 700), RiskLevel::Critical);
    }

    #[test]
    fn test_margin_position() {
        let pos = MarginPosition::new(1000.0, 800.0, 0.85);
        assert_eq!(pos.health_factor(), 1.25);
        assert_eq!(pos.state(), LiquidationState::Safe);

        let pos2 = MarginPosition::new(800.0, 1000.0, 0.85);
        assert_eq!(pos2.state(), LiquidationState::Liquidatable);
    }
}
