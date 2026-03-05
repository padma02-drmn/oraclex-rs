//! OracleX - Oracle Desynchronization Risk Simulator
//!
//! A Rust-based CLI tool for analyzing oracle timing and desynchronization
//! risks in DeFi protocols.

#![allow(dead_code)]
#![allow(clippy::wrong_self_convention)]

mod config;
mod engine;
mod math;
mod oracle;
mod report;
mod types;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use config::SimulationConfig;
use engine::simulator::Simulator;
use report::{generate_json_report, generate_terminal_report, write_report_to_file, ReportFormat};

/// OracleX - Oracle Desynchronization Risk Simulator
#[derive(Parser, Debug)]
#[command(name = "oraclex")]
#[command(author = "Security Researcher")]
#[command(version)]
#[command(about = "Auditor-grade oracle desynchronization risk simulator for DeFi protocols")]
struct Args {
    /// Path to the configuration JSON file
    #[arg(short, long)]
    config: String,

    /// Output file path (optional, defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Output format: json, json-compact, terminal, markdown
    #[arg(short, long, default_value = "terminal")]
    format: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Only show summary (no detailed events)
    #[arg(short, long)]
    summary: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Print banner
    if !args.summary {
        print_banner();
    }

    // Load configuration
    if args.verbose {
        println!("{} Loading configuration from: {}", "→".blue(), args.config);
    }

    let config = SimulationConfig::from_file(&args.config)?;

    if args.verbose {
        println!("{} Configuration loaded:", "✓".green());
        println!("   Price points: {}", config.prices.len());
        println!("   TWAP window:  {}s", config.twap_window_sec);
        println!("   EMA alpha:    {}", config.ema_alpha);
        println!("   Time span:    {}s", config.time_span_sec());
    }

    // Run simulation
    if args.verbose {
        println!("\n{} Running simulation...", "→".blue());
    }

    let mut simulator = Simulator::new(config);
    let result = simulator.run();

    if args.verbose {
        println!(
            "{} Simulation completed in {}ms",
            "✓".green(),
            result.metadata.simulation_duration_ms
        );
    }

    // Generate output
    let format = match args.format.to_lowercase().as_str() {
        "json" => ReportFormat::Json,
        "json-compact" => ReportFormat::JsonCompact,
        "markdown" | "md" => ReportFormat::Markdown,
        _ => ReportFormat::Terminal,
    };

    match &args.output {
        Some(path) => {
            write_report_to_file(&result, path, format)?;
            println!("\n{} Report written to: {}", "✓".green(), path);
        }
        None => {
            // Output to stdout
            let output = match format {
                ReportFormat::Json => generate_json_report(&result),
                ReportFormat::JsonCompact => serde_json::to_string(&result)?,
                ReportFormat::Markdown => report::generate_markdown_report(&result),
                ReportFormat::Terminal => generate_terminal_report(&result),
            };
            println!("{}", output);
        }
    }

    // Exit code based on risk level
    let exit_code = match result.risk_level {
        types::RiskLevel::Critical => 3,
        types::RiskLevel::High => 2,
        types::RiskLevel::Medium => 1,
        types::RiskLevel::Low => 0,
    };

    std::process::exit(exit_code);
}

fn print_banner() {
    println!(
        r#"
{}
   ____                 _     __  __
  / __ \               | |   \ \/ /
 | |  | |_ __ __ _  ___| | ___\  / 
 | |  | | '__/ _` |/ __| |/ _ \/  
 | |__| | | | (_| | (__| |  __/\  
  \____/|_|  \__,_|\___|_|\___| \_\ {}

{}
"#,
        "═".repeat(45).bright_blue(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed(),
        "Oracle Desynchronization Risk Simulator".bright_cyan()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PricePoint;

    #[test]
    fn test_full_simulation_pipeline() {
        let config = SimulationConfig {
            prices: vec![
                PricePoint::new(1000, 100.0),
                PricePoint::new(1060, 105.0),
                PricePoint::new(1120, 103.0),
                PricePoint::new(1180, 110.0),
                PricePoint::new(1240, 108.0),
            ],
            twap_window_sec: 300,
            ema_alpha: 0.1,
            max_oracle_delay_sec: 600,
            liquidation_threshold: 0.85,
            high_deviation_threshold_pct: 2.0,
            high_duration_threshold_sec: 120,
            verbose: false,
            description: Some("Test simulation".to_string()),
        };

        let mut simulator = Simulator::new(config);
        let result = simulator.run();

        // Verify result structure
        assert!(result.metadata.price_points_count == 5);
        assert!(result.max_desync_percent >= 0.0);

        // Verify JSON generation
        let json = generate_json_report(&result);
        assert!(json.contains("risk_level"));

        // Verify terminal report generation
        let term = generate_terminal_report(&result);
        assert!(term.contains("OracleX"));
    }
}
