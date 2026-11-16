//! Portable transcendental functions.
//!
//! `f64::sin`, `cos`, `exp`, `ln` and `powf` call the platform's C maths library,
//! and those libraries disagree in the last bit for some inputs. Measured on
//! 2026-11-15: with the std functions, a native macOS build and the WASM build
//! started from identical state and differed after ONE tick (one position off by
//! one unit in the last place, which then changed mate choice and every later
//! random draw). "Same seed = same run" therefore held only on a single platform.
//!
//! These wrappers use the pure-Rust `libm` crate, which performs the same IEEE-754
//! basic operations on every target. Basic arithmetic, `sqrt`, `abs`, `min`, `max`
//! and `%` are already exactly specified by IEEE-754 and need no wrapper.
//!
//! The same measurement found a second cause: `rng.gen_range(0..len)` with a
//! `usize` length draws 32 bits on 32-bit targets (WASM) and 64 bits on 64-bit
//! ones, so the two consumed the random stream differently. [`random_index`]
//! always draws a `u32`.
//!
//! Every transcendental call and every random index in sim-core must go through
//! this module; `tests/portability.rs` enforces it.

use rand::Rng;

/// Uniform index in `0..len`, drawn as a `u32` on every platform.
/// `len` must be in `1..=u32::MAX` (genome and node lists are far smaller).
#[inline]
pub fn random_index(rng: &mut impl Rng, len: usize) -> usize {
    debug_assert!(len > 0 && len <= u32::MAX as usize);
    rng.gen_range(0..len as u32) as usize
}

#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// Natural logarithm.
#[inline]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

#[inline]
pub fn powf(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
