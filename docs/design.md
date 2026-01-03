# OracleX Design Document

## Overview

OracleX is a Rust-based simulation tool designed for DeFi security researchers to analyze oracle desynchronization risks. This document outlines the architectural decisions and design principles.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        OracleX CLI                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐   │
│   │   Config     │     │   Engine     │     │   Report     │   │
│   │   Parser     │ ──→ │   Simulator  │ ──→ │   Generator  │   │
│   └──────────────┘     └──────────────┘     └──────────────┘   │
│                               │                                  │
│                               ▼                                  │
│          ┌────────────────────────────────────────┐             │
│          │           Oracle Modules               │             │
│          │  ┌────────┐  ┌────────┐  ┌────────┐   │             │
│          │  │  Spot  │  │  TWAP  │  │  EMA   │   │             │
│          │  └────────┘  └────────┘  └────────┘   │             │
│          └────────────────────────────────────────┘             │
│                               │                                  │
│                               ▼                                  │
│          ┌────────────────────────────────────────┐             │
│          │           Metrics Collector            │             │
│          │    • Deviation tracking               │             │
│          │    • Risk classification              │             │
│          │    • Event detection                  │             │
│          └────────────────────────────────────────┘             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Configuration (`config.rs`)

The configuration module handles:
- JSON file parsing
- Input validation
- Default value management

Key fields:
- `prices`: Array of timestamp/price pairs
- `twap_window_sec`: TWAP calculation window
- `ema_alpha`: EMA smoothing factor
- `max_oracle_delay_sec`: Staleness threshold
- `liquidation_threshold`: LTV threshold for simulated positions

### 2. Oracle Implementations (`oracle/`)

#### Spot Oracle
- Returns latest price immediately
- Tracks price history for volatility calculation
- Provides min/max/avg statistics

#### TWAP Oracle
- Implements sliding window average
- Time-weighted calculation
- Automatic pruning of old observations

#### EMA Oracle
- Exponential moving average with configurable alpha
- Lag calculation from spot price
- Convergence time estimation

### 3. Simulation Engine (`engine/`)

The engine orchestrates the simulation:

1. **Initialization**: Create oracles with configuration
2. **Processing Loop**: For each price point:
   - Update all oracles
   - Calculate deviations
   - Check risk conditions
   - Track desync windows
3. **Event Detection**: Identify and record desync events
4. **Result Generation**: Compile statistics and risk assessment

### 4. Metrics (`engine/metrics.rs`)

Statistical functions:
- Mean, standard deviation
- Percentile calculation
- Volatility-adjusted deviation
- Z-score computation
- Time-to-recover estimation

### 5. Report Generation (`report.rs`)

Output formats:
- **JSON**: Machine-readable, complete data
- **Terminal**: Human-readable with colors
- **Markdown**: Documentation-friendly

## Risk Classification

```
Risk Level | Conditions
───────────┼────────────────────────────────────
CRITICAL   │ deviation > 5% OR duration > 600s
HIGH       │ deviation > 2% AND duration > 120s
MEDIUM     │ deviation > 1%
LOW        │ All other cases
```

## Security Considerations

### Non-Goals
- ❌ Trading signals
- ❌ MEV extraction
- ❌ Live exploitation

### Intended Use
- ✅ Security auditing
- ✅ Protocol design validation
- ✅ Risk parameter tuning

## Performance

- **Time Complexity**: O(n) where n = number of price points
- **Space Complexity**: O(w) where w = TWAP window size
- **Typical Runtime**: < 100ms for 1000 price points

## Future Enhancements

1. **Multi-Oracle Correlation**: Analyze relationships between different oracle sources
2. **Flashloan Simulation**: Model flashloan-based manipulation attacks
3. **Chainlink Integration**: Parse real Chainlink update patterns
4. **HTML Reports**: Interactive web-based visualization
