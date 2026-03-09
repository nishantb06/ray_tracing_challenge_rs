/// Epsilon for floating-point comparisons
pub const EPSILON: f64 = 1e-5;

/// Returns true if two floats are equal within EPSILON
pub fn equal(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPSILON
}