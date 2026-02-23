use core::f64;
use std::f64::consts::E;

/// softmax with base
/// https://www.desmos.com/calculator/ri3wwg0ieb
pub fn softmax(base_exp: f64, a: f64, b: f64) -> f64 {
    E.powf(base_exp*a) / (E.powf(base_exp*a) + E.powf(base_exp*b))
}


/// softmax normalised to 0-1 range.
pub fn softmax_diff(base_exp: f64, a: f64, b: f64) -> f64 {
    let x = softmax(base_exp, a, b);

    2.0 * f64::min(x, 1.0 - x)
}