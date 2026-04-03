use core::f64;
use std::f64::consts::E;

/// softmax with base
/// https://www.desmos.com/calculator/ri3wwg0ieb
pub fn softmax(base_exp: f64, a: f64, b: f64) -> f64 {
    let m = a.max(b) * base_exp;
    let ea = (base_exp * a - m).exp();
    let eb = (base_exp * b - m).exp();
    ea / (ea + eb)
}


/// softmax normalised to 0-1 range.
pub fn softmax_diff(base_exp: f64, a: f64, b: f64) -> f64 {
    let x = softmax(base_exp, a, b);

    f64::clamp(2.0 * f64::min(x, 1.0 - x), 0., 1.)
}

#[cfg(test)]
mod tests  {
    use crate::math::softmax;

    #[test]
    fn test_softmax() {
        let s = softmax(0.2, 140961.0, 0.0); // E.powf(0.2*140961.0) / (E.powf(0.2*140961.0) + E.powf(0.2*0))

        dbg!(s);

    }
}
