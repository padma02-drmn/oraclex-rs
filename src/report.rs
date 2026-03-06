//! Report Generation
//!
//! This module provides utilities for generating human-readable
//! and machine-parseable reports from simulation results.

use crate::types::{RiskLevel, SimulationResult};
use colored::Colorize;
use std::io::Write;

/// Generate a JSON report
pub fn generate_json_report(result: &SimulationResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
}

/// Generate a compact JSON report (single line)
pub fn generate_json_compact(result: &SimulationResult) -> String {
    serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string())
}

/// Generate a terminal-friendly report with colors
pub fn generate_terminal_report(result: &SimulationResult) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("\n{}\n", "═".repeat(60).bright_blue()));
    output.push_str(&format!(
        "{}  OracleX Simulation Report\n",
        "🔮"
    ));
    output.push_str(&format!("{}\n\n", "═".repeat(60).bright_blue()));

    // Risk Level (with color)
    let risk_colored = match result.risk_level {
        RiskLevel::Low => result.risk_level.to_string().green(),
        RiskLevel::Medium => result.risk_level.to_string().yellow(),
        RiskLevel::High => result.risk_level.to_string().red(),
        RiskLevel::Critical => result.risk_level.to_string().bright_red().bold(),
    };
    output.push_str(&format!("📊 Overall Risk Level: {}\n\n", risk_colored));

    // Key Metrics
    output.push_str(&format!("{}\n", "─".repeat(40)));
    output.push_str(&format!("{}\n", "📈 Key Metrics".bold()));
    output.push_str(&format!("{}\n", "─".repeat(40)));
    output.push_str(&format!(
        "   Max Desync:       {:.2}%\n",
        result.max_desync_percent
    ));
    output.push_str(&format!(
        "   Desync Duration:  {}s\n",
        result.desync_window_sec
    ));
    output.push_str(&format!(
        "   Worst Timestamp:  {}\n",
        result.worst_timestamp
    ));
    output.push_str(&format!("   Events Detected:  {}\n", result.events.len()));

    // Flags
    if !result.flags.is_empty() {
        output.push_str(&format!("\n{}\n", "─".repeat(40)));
        output.push_str(&format!("{}\n", "⚠️  Risk Flags".bold()));
        output.push_str(&format!("{}\n", "─".repeat(40)));
        for flag in &result.flags {
            output.push_str(&format!("   • {}\n", flag.to_string().yellow()));
        }
    }

    // Oracle Statistics
    output.push_str(&format!("\n{}\n", "─".repeat(40)));
    output.push_str(&format!("{}\n", "📉 Oracle Statistics".bold()));
    output.push_str(&format!("{}\n", "─".repeat(40)));
    output.push_str(&format!(
        "   Spot Updates:     {}\n",
        result.oracle_stats.spot.update_count
    ));
    output.push_str(&format!(
        "   Spot-TWAP Avg:    {:.2}%\n",
        result.oracle_stats.spot_twap_deviation_avg
    ));
    output.push_str(&format!(
        "   Spot-TWAP Max:    {:.2}%\n",
        result.oracle_stats.spot_twap_deviation_max
    ));
    output.push_str(&format!(
        "   Spot-EMA Avg:     {:.2}%\n",
        result.oracle_stats.spot_ema_deviation_avg
    ));

    // Events Summary
    if !result.events.is_empty() {
        output.push_str(&format!("\n{}\n", "─".repeat(40)));
        output.push_str(&format!("{}\n", "📋 Desync Events".bold()));
        output.push_str(&format!("{}\n", "─".repeat(40)));

        for (i, event) in result.events.iter().enumerate().take(5) {
            let risk_marker = match event.risk_level {
                RiskLevel::Low => "🟢",
                RiskLevel::Medium => "🟡",
                RiskLevel::High => "🔴",
                RiskLevel::Critical => "⛔",
            };
            output.push_str(&format!(
                "   {} Event #{}: {:.2}% for {}s ({} -> {})\n",
                risk_marker,
                i + 1,
                event.max_deviation_pct,
                event.duration_sec(),
                event.start_timestamp,
                event.end_timestamp
            ));
        }

        if result.events.len() > 5 {
            output.push_str(&format!(
                "   ... and {} more events\n",
                result.events.len() - 5
            ));
        }
    }

    // Metadata
    output.push_str(&format!("\n{}\n", "─".repeat(40)));
    output.push_str(&format!("{}\n", "ℹ️  Metadata".bold()));
    output.push_str(&format!("{}\n", "─".repeat(40)));
    output.push_str(&format!(
        "   Version:          v{}\n",
        result.metadata.version
    ));
    output.push_str(&format!(
        "   Price Points:     {}\n",
        result.metadata.price_points_count
    ));
    output.push_str(&format!(
        "   Simulation Time:  {}ms\n",
        result.metadata.simulation_duration_ms
    ));
    output.push_str(&format!(
        "   Config:           {}\n",
        result.metadata.config_summary
    ));

    output.push_str(&format!("\n{}\n", "═".repeat(60).bright_blue()));

    output
}

/// Write report to a file
pub fn write_report_to_file(
    result: &SimulationResult,
    path: &str,
    format: ReportFormat,
) -> std::io::Result<()> {
    let content = match format {
        ReportFormat::Json => generate_json_report(result),
        ReportFormat::JsonCompact => generate_json_compact(result),
        ReportFormat::Terminal => generate_terminal_report(result),
        ReportFormat::Markdown => generate_markdown_report(result),
    };

    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Report format options
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    /// Pretty-printed JSON
    Json,
    /// Compact JSON (single line)
    JsonCompact,
    /// Terminal-friendly with colors
    Terminal,
    /// Markdown format
    Markdown,
}

/// Generate a Markdown report
pub fn generate_markdown_report(result: &SimulationResult) -> String {
    let mut output = String::new();

    output.push_str("# OracleX Simulation Report\n\n");

    // Summary Table
    output.push_str("## Summary\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Risk Level | **{}** |\n", result.risk_level));
    output.push_str(&format!(
        "| Max Desync | {:.2}% |\n",
        result.max_desync_percent
    ));
    output.push_str(&format!(
        "| Desync Duration | {}s |\n",
        result.desync_window_sec
    ));
    output.push_str(&format!("| Events Detected | {} |\n", result.events.len()));
    output.push_str(&format!(
        "| Price Points | {} |\n\n",
        result.metadata.price_points_count
    ));

    // Risk Flags
    if !result.flags.is_empty() {
        output.push_str("## Risk Flags\n\n");
        for flag in &result.flags {
            output.push_str(&format!("- ⚠️ `{}`\n", flag));
        }
        output.push('\n');
    }

    // Events Table
    if !result.events.is_empty() {
        output.push_str("## Desync Events\n\n");
        output.push_str("| # | Start | End | Duration | Deviation | Risk |\n");
        output.push_str("|---|-------|-----|----------|-----------|------|\n");

        for (i, event) in result.events.iter().enumerate() {
            output.push_str(&format!(
                "| {} | {} | {} | {}s | {:.2}% | {} |\n",
                i + 1,
                event.start_timestamp,
                event.end_timestamp,
                event.duration_sec(),
                event.max_deviation_pct,
                event.risk_level
            ));
        }
        output.push('\n');
    }

    // Oracle Statistics
    output.push_str("## Oracle Statistics\n\n");
    output.push_str(&format!(
        "- **Spot Updates:** {}\n",
        result.oracle_stats.spot.update_count
    ));
    output.push_str(&format!(
        "- **Spot-TWAP Deviation (avg):** {:.2}%\n",
        result.oracle_stats.spot_twap_deviation_avg
    ));
    output.push_str(&format!(
        "- **Spot-TWAP Deviation (max):** {:.2}%\n",
        result.oracle_stats.spot_twap_deviation_max
    ));
    output.push_str(&format!(
        "- **Spot-EMA Deviation (avg):** {:.2}%\n\n",
        result.oracle_stats.spot_ema_deviation_avg
    ));

    // Footer
    output.push_str("---\n\n");
    output.push_str(&format!(
        "*Generated by OracleX v{} | Simulation time: {}ms*\n",
        result.metadata.version, result.metadata.simulation_duration_ms
    ));

    output
}

/// Print result summary to stdout
pub fn print_summary(result: &SimulationResult) {
    println!("{}", generate_terminal_report(result));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DesyncEvent, DesyncFlag, OracleStatsSummary, SimulationMetadata};

    fn create_test_result() -> SimulationResult {
        SimulationResult {
            max_desync_percent: 2.5,
            desync_window_sec: 180,
            worst_timestamp: 1700000000,
            risk_level: RiskLevel::High,
            flags: vec![DesyncFlag::SpotTwapDivergence, DesyncFlag::HighVolatility],
            events: vec![DesyncEvent {
                start_timestamp: 1700000000,
                end_timestamp: 1700000180,
                max_deviation_pct: 2.5,
                oracle_pair: "spot-twap".to_string(),
                risk_level: RiskLevel::High,
                flags: vec![DesyncFlag::SpotTwapDivergence],
            }],
            oracle_stats: OracleStatsSummary::default(),
            metadata: SimulationMetadata::default(),
        }
    }

    #[test]
    fn test_json_report() {
        let result = create_test_result();
        let json = generate_json_report(&result);

        assert!(json.contains("max_desync_percent"));
        assert!(json.contains("2.5"));
        assert!(json.contains("HIGH"));
    }

    #[test]
    fn test_markdown_report() {
        let result = create_test_result();
        let md = generate_markdown_report(&result);

        assert!(md.contains("# OracleX Simulation Report"));
        assert!(md.contains("| Risk Level |"));
        assert!(md.contains("HIGH"));
    }

    #[test]
    fn test_terminal_report() {
        let result = create_test_result();
        let term = generate_terminal_report(&result);

        assert!(term.contains("OracleX Simulation Report"));
        assert!(term.contains("Key Metrics"));
    }
}
