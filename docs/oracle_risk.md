# Oracle Risk Analysis

## Understanding Oracle Desynchronization

Oracle desynchronization occurs when different price sources (or smoothed versions of prices) diverge from each other. This divergence can create exploitable windows in DeFi protocols.

## Types of Oracle Desync

### 1. Spot vs TWAP Divergence

**What it is:** The difference between the current market price (spot) and the time-weighted average price.

**When it occurs:**
- Rapid price movements
- Flash crashes or pumps
- Low liquidity periods

**Risk:** TWAP lags behind spot, creating windows where:
- Users can borrow more than they should (spot > TWAP)
- Users get liquidated unfairly (spot < TWAP)

### 2. Spot vs EMA Divergence

**What it is:** The difference between current price and the exponentially smoothed price.

**When it occurs:**
- Any price volatility
- More pronounced with lower alpha values

**Risk:** Similar to TWAP, but EMA responds faster to recent changes.

### 3. TWAP vs EMA Divergence

**What it is:** Different smoothing algorithms responding differently to the same price data.

**When it occurs:**
- During trend reversals
- Period of high-then-low volatility

**Risk:** Protocols using different smoothing may value the same collateral differently.

## Real-World Oracle Attacks

### Case Study 1: Flash Loan + TWAP Manipulation

1. Attacker takes flash loan
2. Dumps large amount on DEX, crashing spot price
3. TWAP hasn't updated yet (still shows high price)
4. Attacker borrows against "inflated" TWAP value
5. Spot price returns to normal after flash
6. Attacker has borrowed more than collateral value

### Case Study 2: Stale Price Exploitation

1. Oracle updates stop (congestion, bug, etc.)
2. Market price moves significantly
3. Protocol still uses stale price
4. Attacker exploits the difference

### Case Study 3: Oracle Heartbeat Abuse

1. Chainlink has heartbeat (max update interval)
2. Attacker waits until just before expected update
3. Price has drifted significantly since last update
4. Brief window of exploitation exists

## Risk Metrics Explained

### Maximum Deviation

```
max_deviation = max(|spot - oracle| / oracle) × 100%
```

Higher values indicate greater potential for exploitation.

### Desync Window Duration

Time period where deviation exceeds threshold.

Longer windows = more opportunity for attackers.

### Time-to-Recover

Estimated time for smoothed oracle to converge back to spot.

```
T = log(threshold / current_deviation) / log(1 - alpha)
```

### Volatility-Adjusted Deviation

```
adjusted = deviation / historical_volatility
```

A 2% deviation during 0.5% daily volatility is more concerning than during 5% daily volatility.

## Detection Strategies with OracleX

### 1. Threshold Analysis

```bash
# Run with your config
oraclex --config prices.json

# Check for HIGH or CRITICAL risk
```

### 2. Parameter Sensitivity

Try different TWAP windows and EMA alphas:

```json
// Test 1: 5-minute window
{ "twap_window_sec": 300 }

// Test 2: 30-minute window  
{ "twap_window_sec": 1800 }
```

Compare risk levels to find optimal parameters.

### 3. Stress Testing

Create synthetic price data with extreme scenarios:

```json
{
  "prices": [
    { "timestamp": 0, "price": 100.0 },
    { "timestamp": 60, "price": 50.0 },  // 50% flash crash
    { "timestamp": 120, "price": 100.0 } // Recovery
  ]
}
```

## Mitigation Strategies

### For Protocol Developers

1. **Multi-Oracle Validation**: Require price agreement across sources
2. **Bounds Checking**: Reject prices outside sanity bounds
3. **Delay Sensitive Operations**: Add block delay for large actions
4. **Circuit Breakers**: Pause when deviation exceeds threshold

### For Auditors

1. **Identify Oracle Usage**: Find all price consumption points
2. **Check Update Frequency**: Verify heartbeat/staleness checks
3. **Simulate Edge Cases**: Use tools like OracleX
4. **Review Smoothing Parameters**: Are they appropriate for the use case?

## OracleX Flags Explained

| Flag | Meaning | Risk |
|------|---------|------|
| `STALE_PRICE_WINDOW` | Oracle hasn't updated in max_delay time | Old price exploitation |
| `FALSE_LIQUIDATION_RISK` | Could liquidate when shouldn't | User fund loss |
| `ESCAPED_LIQUIDATION` | Should liquidate but doesn't | Protocol insolvency |
| `SPOT_TWAP_DIVERGENCE` | Large spot-TWAP gap | Various attacks |
| `HIGH_VOLATILITY` | Extreme price movements | Increased risk |
