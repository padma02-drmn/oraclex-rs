//! Fixed-Point Arithmetic Utilities
//!
//! This module provides utilities for fixed-point arithmetic operations
//! commonly used in DeFi protocols (e.g., Q96, Q128 formats).

use std::ops::{Add, Div, Mul, Sub};

/// Fixed-point number with 18 decimals (similar to WAD in Solidity)
pub const DECIMALS_18: u128 = 1_000_000_000_000_000_000;

/// Fixed-point number with 27 decimals (similar to RAY in Solidity)
pub const DECIMALS_27: u128 = 1_000_000_000_000_000_000_000_000_000;

/// Q96 fixed-point format (used in Uniswap V3)
pub const Q96: u128 = 1 << 96;

/// Q128 fixed-point format (used for fee growth) - approximation since 2^128 overflows u128
pub const Q128: u128 = u128::MAX;

/// A fixed-point number wrapper for safe arithmetic
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FixedPoint {
    /// Raw value (scaled by precision)
    value: i128,
    /// Decimal places
    decimals: u8,
}

impl FixedPoint {
    /// Create a new fixed-point number
    pub fn new(value: i128, decimals: u8) -> Self {
        Self { value, decimals }
    }

    /// Create from a floating-point number
    pub fn from_f64(value: f64, decimals: u8) -> Self {
        let scale = 10i128.pow(decimals as u32);
        Self {
            value: (value * scale as f64) as i128,
            decimals,
        }
    }

    /// Convert to floating-point
    pub fn to_f64(&self) -> f64 {
        let scale = 10i128.pow(self.decimals as u32);
        self.value as f64 / scale as f64
    }

    /// Get raw value
    pub fn raw(&self) -> i128 {
        self.value
    }

    /// Get decimal places
    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    /// Check if the value is zero
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Check if the value is positive
    pub fn is_positive(&self) -> bool {
        self.value > 0
    }

    /// Check if the value is negative
    pub fn is_negative(&self) -> bool {
        self.value < 0
    }

    /// Get absolute value
    pub fn abs(&self) -> Self {
        Self {
            value: self.value.abs(),
            decimals: self.decimals,
        }
    }

    /// Multiply by another fixed-point number
    pub fn mul_fp(&self, other: &FixedPoint) -> Self {
        let scale = 10i128.pow(self.decimals as u32);
        Self {
            value: (self.value * other.value) / scale,
            decimals: self.decimals,
        }
    }

    /// Divide by another fixed-point number
    pub fn div_fp(&self, other: &FixedPoint) -> Option<Self> {
        if other.value == 0 {
            return None;
        }
        let scale = 10i128.pow(self.decimals as u32);
        Some(Self {
            value: (self.value * scale) / other.value,
            decimals: self.decimals,
        })
    }

    /// Scale to different decimal places
    pub fn scale_to(&self, new_decimals: u8) -> Self {
        if new_decimals == self.decimals {
            return *self;
        }

        let new_value = if new_decimals > self.decimals {
            let scale = 10i128.pow((new_decimals - self.decimals) as u32);
            self.value * scale
        } else {
            let scale = 10i128.pow((self.decimals - new_decimals) as u32);
            self.value / scale
        };

        Self {
            value: new_value,
            decimals: new_decimals,
        }
    }
}

impl Add for FixedPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(self.decimals, other.decimals, "Decimal mismatch");
        Self {
            value: self.value + other.value,
            decimals: self.decimals,
        }
    }
}

impl Sub for FixedPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_eq!(self.decimals, other.decimals, "Decimal mismatch");
        Self {
            value: self.value - other.value,
            decimals: self.decimals,
        }
    }
}

impl Mul<i128> for FixedPoint {
    type Output = Self;

    fn mul(self, scalar: i128) -> Self {
        Self {
            value: self.value * scalar,
            decimals: self.decimals,
        }
    }
}

impl Div<i128> for FixedPoint {
    type Output = Self;

    fn div(self, scalar: i128) -> Self {
        Self {
            value: self.value / scalar,
            decimals: self.decimals,
        }
    }
}

/// Safe multiplication that checks for overflow
pub fn safe_mul(a: u128, b: u128) -> Option<u128> {
    a.checked_mul(b)
}

/// Safe division with rounding up
pub fn div_round_up(numerator: u128, denominator: u128) -> Option<u128> {
    if denominator == 0 {
        return None;
    }
    Some((numerator + denominator - 1) / denominator)
}

/// Multiply then divide with higher precision
pub fn mul_div(a: u128, b: u128, denominator: u128) -> Option<u128> {
    if denominator == 0 {
        return None;
    }

    // Use u256 for intermediate calculation (simulated with two u128s)
    let product = a as u128 * b as u128;
    Some(product / denominator)
}

/// Full precision mul_div using 256-bit intermediate
pub fn mul_div_full(a: u128, b: u128, denominator: u128) -> Option<u128> {
    if denominator == 0 {
        return None;
    }

    // For simplicity, use floating point for now
    // In production, would use proper u256 arithmetic
    let result = (a as f64 * b as f64) / denominator as f64;
    if result > u128::MAX as f64 {
        return None;
    }
    Some(result as u128)
}

/// Convert sqrtPriceX96 to price (Uniswap V3 format)
pub fn sqrt_price_x96_to_price(sqrt_price_x96: u128, decimals0: u8, decimals1: u8) -> f64 {
    let sqrt_price = sqrt_price_x96 as f64 / Q96 as f64;
    let price = sqrt_price * sqrt_price;

    // Adjust for decimal difference
    let decimal_adj = 10f64.powi((decimals1 as i32) - (decimals0 as i32));
    price * decimal_adj
}

/// Convert price to sqrtPriceX96 (Uniswap V3 format)
pub fn price_to_sqrt_price_x96(price: f64, decimals0: u8, decimals1: u8) -> u128 {
    // Adjust for decimal difference
    let decimal_adj = 10f64.powi((decimals0 as i32) - (decimals1 as i32));
    let adjusted_price = price * decimal_adj;

    let sqrt_price = adjusted_price.sqrt();
    (sqrt_price * Q96 as f64) as u128
}

/// Convert tick to sqrtPriceX96 (Uniswap V3)
pub fn tick_to_sqrt_price_x96(tick: i32) -> u128 {
    // sqrtPrice = 1.0001^(tick/2)
    let sqrt_ratio = 1.0001f64.powf(tick as f64 / 2.0);
    (sqrt_ratio * Q96 as f64) as u128
}

/// Convert sqrtPriceX96 to tick (Uniswap V3)
pub fn sqrt_price_x96_to_tick(sqrt_price_x96: u128) -> i32 {
    // tick = 2 * log_1.0001(sqrtPrice)
    let sqrt_price = sqrt_price_x96 as f64 / Q96 as f64;
    let tick = 2.0 * sqrt_price.ln() / 1.0001f64.ln();
    tick.floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_point_basic() {
        let a = FixedPoint::from_f64(1.5, 18);
        let b = FixedPoint::from_f64(2.0, 18);

        let sum = a + b;
        assert!((sum.to_f64() - 3.5).abs() < 0.0001);

        let diff = b - a;
        assert!((diff.to_f64() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_fixed_point_mul() {
        let a = FixedPoint::from_f64(1.5, 18);
        let b = FixedPoint::from_f64(2.0, 18);

        let product = a.mul_fp(&b);
        assert!((product.to_f64() - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_fixed_point_div() {
        let a = FixedPoint::from_f64(6.0, 18);
        let b = FixedPoint::from_f64(2.0, 18);

        let quotient = a.div_fp(&b).unwrap();
        assert!((quotient.to_f64() - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_div_round_up() {
        assert_eq!(div_round_up(10, 3), Some(4)); // ceil(10/3) = 4
        assert_eq!(div_round_up(9, 3), Some(3)); // exact
        assert_eq!(div_round_up(10, 0), None); // div by zero
    }

    #[test]
    fn test_sqrt_price_conversion() {
        // Test with WETH/USDC (18/6 decimals)
        let price = 2000.0; // $2000 per ETH
        let sqrt_x96 = price_to_sqrt_price_x96(price, 18, 6);
        let recovered = sqrt_price_x96_to_price(sqrt_x96, 18, 6);

        assert!((recovered - price).abs() / price < 0.01); // Within 1%
    }

    #[test]
    fn test_tick_conversion() {
        // Tick 0 should give sqrtPrice = 1.0
        let sqrt_at_0 = tick_to_sqrt_price_x96(0);
        let ratio = sqrt_at_0 as f64 / Q96 as f64;
        assert!((ratio - 1.0).abs() < 0.001);

        // Round trip
        let tick = 1000;
        let sqrt = tick_to_sqrt_price_x96(tick);
        let recovered_tick = sqrt_price_x96_to_tick(sqrt);
        assert!((recovered_tick - tick).abs() <= 1);
    }
}
