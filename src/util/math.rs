// Returns decimal percentage
pub fn calculate_pct_difference(execution_price: f64, mid_price: f64) -> f64 {
    (execution_price - mid_price).abs() / mid_price
}

pub fn calculate_effective_rate(rate1: f64, rate2: f64) -> f64 {
    if rate1.signum() != rate2.signum() {
        rate1.abs() + rate2.abs()
    } else {
        rate1.abs().max(rate2.abs()) - rate1.abs().min(rate2.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_pct_diff() {
        let buying = calculate_pct_difference(102.0, 100.0); // buying
        let selling = calculate_pct_difference(100.0, 102.0); // selling
        assert_relative_eq!(buying, 0.02, max_relative = 0.001);
        println!("buying pct diff: {buying}");
        assert_relative_eq!(selling, 0.0196, max_relative = 0.001);
        println!("selling pct diff: {selling}");
    }

    #[test]
    fn test_calculate_effective_rate() {
        assert_eq!(calculate_effective_rate(0.001, -0.002), 0.003);
        assert_eq!(calculate_effective_rate(-0.001, 0.002), 0.003);
        assert_eq!(calculate_effective_rate(0.001, 0.002), 0.001);
        assert_eq!(calculate_effective_rate(-0.001, -0.002), 0.001);
    }
}
