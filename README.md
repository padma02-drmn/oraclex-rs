# 🔮 OracleX

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Security](https://img.shields.io/badge/focus-DeFi%20Security-red.svg)](https://github.com/yourusername/oraclex-rs)

> **Auditor-grade Oracle Desynchronization Risk Simulator for DeFi Protocols**

OracleX is a Rust-based command-line tool that simulates and analyzes oracle desynchronization risks in DeFi protocols. It helps security researchers identify potential liquidation risks, price manipulation windows, and oracle timing vulnerabilities.

## 🎯 Features

- **Multi-Oracle Simulation**: Spot, TWAP, and EMA oracle implementations
- **Desync Detection**: Identifies dangerous price divergence windows
- **Risk Quantification**: Calculates liquidation and solvency risks
- **Auditor Reports**: JSON and terminal output for audit documentation

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/oraclex-rs.git
cd oraclex-rs

# Build the project
cargo build --release

# Run with sample data
cargo run -- --config examples/simple_eth_oracle.json
```

### Basic Usage

```bash
# Analyze oracle data from JSON config
oraclex --config config.json

# Output to file
oraclex --config config.json --output report.json

# Verbose mode
oraclex --config config.json --verbose
```

## 📊 What This Tool Analyzes

### 1️⃣ Oracle Timing
- Timestamp drift between updates
- Update frequency mismatches
- Stale price window detection
- Maximum delay threshold breaches

### 2️⃣ Oracle Type Desync
- Spot vs TWAP divergence
- Spot vs EMA lag analysis
- TWAP vs EMA smoothing gaps

### 3️⃣ Price Deviation
- Absolute deviation (%)
- Log-return deviation
- Volatility-adjusted deviation

### 4️⃣ Liquidation Risk (Simulated)
- Margin threshold crossing detection
- False liquidation window identification
- Escaped liquidation scenarios

## 📁 Input Format

```json
{
  "prices": [
    { "timestamp": 1700000000, "price": 100.0 },
    { "timestamp": 1700000060, "price": 101.2 },
    { "timestamp": 1700000120, "price": 99.8 }
  ],
  "twap_window_sec": 300,
  "ema_alpha": 0.1,
  "max_oracle_delay_sec": 600,
  "liquidation_threshold": 0.85
}
```

## 📋 Output Format

```json
{
  "max_desync_percent": 2.14,
  "desync_window_sec": 180,
  "worst_timestamp": 1700000420,
  "risk_level": "HIGH",
  "flags": [
    "FALSE_LIQUIDATION_RISK",
    "STALE_PRICE_WINDOW"
  ],
  "oracle_stats": {
    "spot_updates": 100,
    "twap_deviation_avg": 0.45,
    "ema_lag_max": 1.2
  }
}
```

## 🔴 Risk Classification

| Condition | Risk Level |
|-----------|------------|
| Deviation > 5% OR Duration > 600s | CRITICAL |
| Deviation > 2% AND Duration > 120s | HIGH |
| Deviation > 1% | MEDIUM |
| Otherwise | LOW |

## 🏗️ Architecture

```
oraclex-rs/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── config.rs         # Configuration parsing
│   ├── types.rs          # Core data structures
│   ├── report.rs         # Report generation
│   ├── oracle/           # Oracle implementations
│   │   ├── mod.rs
│   │   ├── spot.rs       # Spot price oracle
│   │   ├── twap.rs       # Time-weighted average
│   │   └── ema.rs        # Exponential moving average
│   ├── engine/           # Simulation engine
│   │   ├── mod.rs
│   │   ├── simulator.rs  # Core simulation loop
│   │   └── metrics.rs    # Risk calculations
│   └── math/             # Math utilities
│       ├── mod.rs
│       └── fixed.rs      # Fixed-point arithmetic
├── examples/             # Sample configurations
├── docs/                 # Documentation
└── Cargo.toml
```

## 🔒 Non-Goals

This tool is designed for **security analysis only**:

- ❌ No trading signals
- ❌ No MEV extraction
- ❌ No live exploitation
- ✅ Pure analysis & simulation

## 📚 Documentation

- [Design Document](docs/design.md)
- [Oracle Risk Analysis](docs/oracle_risk.md)
- [Usage Examples](docs/examples.md)

## 🤝 Use Cases

1. **Security Audits**: Analyze oracle assumptions in DeFi protocols
2. **Protocol Design**: Test oracle parameters before deployment
3. **Risk Assessment**: Quantify liquidation risks under various scenarios
4. **Assumption Testing**: Validate claims like "TWAP prevents manipulation"

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built with insights from auditing real DeFi protocols including Panoptic, Aave, and Compound.

---

**Built for DeFi Security Researchers** 🛡️
