# OracleX Usage Examples

This document provides practical examples of using OracleX for oracle risk analysis.

## Basic Usage

### Running with a Configuration File

```bash
# Build the project
cargo build --release

# Run with sample configuration
cargo run -- --config examples/simple_eth_oracle.json
```

### Output Formats

```bash
# Terminal output (default, with colors)
oraclex --config config.json

# JSON output
oraclex --config config.json --format json

# Compact JSON (single line)
oraclex --config config.json --format json-compact

# Markdown output
oraclex --config config.json --format markdown

# Save to file
oraclex --config config.json --output report.json --format json
```

### Verbose Mode

```bash
# See detailed processing information
oraclex --config config.json --verbose
```

## Example Configurations

### Stable Market (Low Risk)

```json
{
  "prices": [
    { "timestamp": 1700000000, "price": 2000.0 },
    { "timestamp": 1700000060, "price": 2001.0 },
    { "timestamp": 1700000120, "price": 2002.0 },
    { "timestamp": 1700000180, "price": 2001.5 },
    { "timestamp": 1700000240, "price": 2000.5 }
  ],
  "twap_window_sec": 300,
  "ema_alpha": 0.1
}
```

Expected output: `Risk Level: LOW`

### Flash Crash Scenario (High Risk)

```json
{
  "prices": [
    { "timestamp": 1700000000, "price": 2000.0 },
    { "timestamp": 1700000060, "price": 1900.0 },
    { "timestamp": 1700000120, "price": 1700.0 },
    { "timestamp": 1700000180, "price": 1500.0 },
    { "timestamp": 1700000240, "price": 1800.0 },
    { "timestamp": 1700000300, "price": 1950.0 }
  ],
  "twap_window_sec": 300,
  "ema_alpha": 0.1
}
```

Expected output: `Risk Level: HIGH` with `SPOT_TWAP_DIVERGENCE` flag

### Slow EMA Response (Testing Lag)

```json
{
  "prices": [
    { "timestamp": 1700000000, "price": 100.0 },
    { "timestamp": 1700000060, "price": 200.0 },
    { "timestamp": 1700000120, "price": 200.0 },
    { "timestamp": 1700000180, "price": 200.0 },
    { "timestamp": 1700000240, "price": 200.0 }
  ],
  "twap_window_sec": 60,
  "ema_alpha": 0.05
}
```

This tests EMA lag when price doubles. With alpha=0.05, EMA converges slowly.

## Interpreting Results

### Sample Terminal Output

```
═══════════════════════════════════════════════════════════
🔮  OracleX Simulation Report
═══════════════════════════════════════════════════════════

📊 Overall Risk Level: HIGH

────────────────────────────────────
📈 Key Metrics
────────────────────────────────────
   Max Desync:       5.23%
   Desync Duration:  180s
   Worst Timestamp:  1700000180
   Events Detected:  2

────────────────────────────────────
⚠️  Risk Flags
────────────────────────────────────
   • SPOT_TWAP_DIVERGENCE
   • HIGH_VOLATILITY
```

### Understanding Risk Levels

| Risk Level | Exit Code | Action |
|------------|-----------|--------|
| LOW (🟢) | 0 | Normal operation |
| MEDIUM (🟡) | 1 | Review parameters |
| HIGH (🔴) | 2 | Investigate further |
| CRITICAL (⛔) | 3 | Immediate attention |

### Integration with CI/CD

```bash
#!/bin/bash
oraclex --config production_params.json --format json-compact

EXIT_CODE=$?

if [ $EXIT_CODE -ge 2 ]; then
    echo "ALERT: High oracle desync risk detected!"
    exit 1
fi

echo "Oracle parameters validated"
exit 0
```

## Advanced Examples

### Comparing TWAP Windows

Create multiple configs and compare:

```bash
# 5-minute TWAP
oraclex --config twap_5m.json --output report_5m.json --format json

# 15-minute TWAP
oraclex --config twap_15m.json --output report_15m.json --format json

# Compare max_desync_percent from both reports
```

### Generating Test Data

Simple Python script to generate test prices:

```python
import json
import random

prices = []
base_price = 2000.0
timestamp = 1700000000

for i in range(100):
    # Random walk
    base_price *= 1 + random.gauss(0, 0.01)  # 1% std dev
    prices.append({
        "timestamp": timestamp,
        "price": round(base_price, 2)
    })
    timestamp += 60  # 1 minute intervals

config = {
    "prices": prices,
    "twap_window_sec": 300,
    "ema_alpha": 0.1
}

with open("random_walk.json", "w") as f:
    json.dump(config, f, indent=2)
```

### Batch Analysis

```bash
#!/bin/bash
for config in configs/*.json; do
    echo "Analyzing: $config"
    oraclex --config "$config" --format json-compact >> results.jsonl
done
```

## Troubleshooting

### "Price data cannot be empty"

Ensure your config has at least 2 price points:

```json
{
  "prices": [
    { "timestamp": 1000, "price": 100.0 },
    { "timestamp": 2000, "price": 101.0 }
  ]
}
```

### "Timestamps must be strictly increasing"

Check that each timestamp is greater than the previous:

```json
// ❌ Wrong
{ "timestamp": 1000, "price": 100.0 },
{ "timestamp": 1000, "price": 101.0 }  // Same timestamp!

// ✅ Correct
{ "timestamp": 1000, "price": 100.0 },
{ "timestamp": 1060, "price": 101.0 }
```

### "EMA alpha must be in range (0, 1]"

Alpha must be between 0 (exclusive) and 1 (inclusive):

```json
// ❌ Wrong
{ "ema_alpha": 0 }
{ "ema_alpha": 1.5 }

// ✅ Correct
{ "ema_alpha": 0.1 }
{ "ema_alpha": 1.0 }
```
