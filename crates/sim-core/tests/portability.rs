//! Guards for cross-platform reproducibility.
//!
//! Measured on 2026-11-15: a native macOS build and the WASM build diverged after
//! one tick, for two reasons. (1) `f64::sin`/`cos`/`exp`/`ln`/`powf` call the
//! platform maths library, whose last-bit results differ. (2) `gen_range` over a
//! `usize` range draws 32 bits on 32-bit targets and 64 bits on 64-bit ones.
//! `sim_core::math` now provides portable versions of both. These tests keep new
//! code from reintroducing either pattern. The cross-platform comparison itself
//! runs in CI (`sim-runner fingerprint` against `web/demo/tests/conformance.mjs`).
use std::fs;
use std::path::Path;

fn core_sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            out.push((name, fs::read_to_string(&path).unwrap()));
        }
    }
    assert!(out.len() > 5, "expected to scan the sim-core sources");
    out
}

/// Code lines only: strips `//` comments so prose may mention the patterns.
fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line.split("//").next().unwrap_or("")))
}

#[test]
fn no_platform_transcendentals_in_core() {
    const FORBIDDEN: [&str; 19] = [
        ".sin()",
        ".cos()",
        ".tan()",
        ".asin()",
        ".acos()",
        ".atan()",
        ".atan2(",
        ".sinh()",
        ".cosh()",
        ".tanh()",
        ".exp()",
        ".exp2()",
        ".exp_m1()",
        ".ln()",
        ".ln_1p()",
        ".log10()",
        ".log2()",
        ".powf(",
        ".cbrt()",
    ];
    let mut hits = Vec::new();
    for (name, text) in core_sources() {
        if name == "math.rs" {
            continue;
        }
        for (line_no, code) in code_lines(&text) {
            for pattern in FORBIDDEN {
                if code.contains(pattern) {
                    hits.push(format!("{name}:{line_no}: `{pattern}`"));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "platform maths functions differ between targets; use sim_core::math instead:\n{}",
        hits.join("\n")
    );
}

#[test]
fn no_pointer_width_random_draws_in_core() {
    let mut hits = Vec::new();
    for (name, text) in core_sources() {
        if name == "math.rs" {
            continue;
        }
        for (line_no, code) in code_lines(&text) {
            let sized_range = code.contains("gen_range(")
                && (code.contains(".len()") || code.contains("usize") || code.contains("isize"));
            let sized_value = code.contains("gen::<usize>") || code.contains("gen::<isize>");
            if sized_range || sized_value {
                hits.push(format!("{name}:{line_no}: {}", code.trim()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "usize random draws consume the RNG differently on 32- and 64-bit targets; \
         use sim_core::math::random_index:\n{}",
        hits.join("\n")
    );
}

/// The portable functions must agree with the mathematical values closely, so
/// swapping them in did not change the model, only the last bits.
#[test]
fn portable_math_matches_reference_values() {
    use sim_core::math;
    let close = |a: f64, b: f64| (a - b).abs() <= 4.0 * f64::EPSILON * b.abs().max(1.0);
    assert!(close(math::sin(0.5), 0.479_425_538_604_203));
    assert!(close(math::cos(0.5), 0.877_582_561_890_372_7));
    assert!(close(math::exp(1.0), std::f64::consts::E));
    assert!(close(math::ln(10.0), std::f64::consts::LN_10));
    assert!(close(math::powf(2.0, 0.75), 1.681_792_830_507_429));
    assert_eq!(math::sin(0.0), 0.0);
    assert_eq!(math::cos(0.0), 1.0);
    assert_eq!(math::exp(0.0), 1.0);
    assert_eq!(math::ln(1.0), 0.0);
}

/// Known-answer test: the same seed must give the same indices on every
/// platform. A 64-bit-only or 32-bit-only draw would produce different values.
#[test]
fn random_index_is_identical_on_every_platform() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(9);
    let drawn: Vec<usize> = [1usize, 2, 7, 231, 512, 300, 35, 64]
        .iter()
        .map(|&len| {
            let idx = sim_core::math::random_index(&mut rng, len);
            assert!(idx < len);
            idx
        })
        .collect();
    assert_eq!(drawn, EXPECTED_INDICES);
}

const EXPECTED_INDICES: [usize; 8] = [0, 1, 1, 198, 490, 144, 15, 32];

/// Positions stay inside [0, size) even for remainders too small to survive
/// the addition of `size`.
#[test]
fn wrap_coord_never_returns_the_upper_bound() {
    use sim_core::world::wrap_coord;
    for v in [
        -1e-17,
        -f64::MIN_POSITIVE,
        -5e-324,
        -1e-13,
        -1000.0,
        -2000.0 - 1e-13,
    ] {
        let w = wrap_coord(v, 1000.0);
        assert!((0.0..1000.0).contains(&w), "wrap_coord({v}) = {w}");
    }
    assert_eq!(wrap_coord(1000.0, 1000.0), 0.0);
    assert_eq!(wrap_coord(-0.5, 1000.0), 999.5);
    assert_eq!(wrap_coord(250.25, 1000.0), 250.25);
}
