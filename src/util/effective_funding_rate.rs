use crate::compare_funding_rates::JointFundingRate;
use crate::util::Platform;

pub fn determine_short_based_on_fr(jfr: JointFundingRate) -> Platform {
    if jfr.binance_funding_rate > jfr.hyperliquid_funding_rate {
        Platform::Binance
    } else {
        Platform::Hyperliquid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_short_long_based_on_fr() {
        let jfr = JointFundingRate {
            name: "".to_string(),
            hyperliquid_open_interest: 0.0,
            binance_mark_price: 0.0,
            funding_rate_difference: 0.0,
            binance_funding_rate: 4.0,
            hyperliquid_funding_rate: -3.0,
        };

        let short = determine_short_based_on_fr(jfr);

        assert_eq!(short, Platform::Binance);
    }
}
