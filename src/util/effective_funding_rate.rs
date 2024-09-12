pub fn calculate_effective_rate(rate1: f64, rate2: f64) -> f64 {
    if rate1.signum() != rate2.signum() {
        rate1.abs() + rate2.abs()
    } else {
        rate1.abs().max(rate2.abs()) - rate1.abs().min(rate2.abs())
    }
}
