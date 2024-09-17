use crate::binance::funding_rates::retrieve_binance_hourly_funding_rates;
use crate::hyperliquid::funding_rates::retrieve_hl_hourly_funding_rates;
use crate::util::calculate_effective_rate;
use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;
use tokio::try_join;

#[derive(Debug)]
pub struct JointFundingRate {
    pub name: String,
    pub binance_funding_rate: f64, // hourly fr decimal form
    pub binance_mark_price: f64,
    pub hyperliquid_funding_rate: f64,  // hourly fr decimal form
    pub hyperliquid_open_interest: f64, // expressed in USD
    pub funding_rate_difference: f64,
}

pub async fn compare_funding_rates() -> Result<Vec<JointFundingRate>> {
    let info_client = InfoClient::new(None, None).await?;

    let (binance_tokens, hyperliquid_tokens) = try_join!(
        retrieve_binance_hourly_funding_rates(),
        retrieve_hl_hourly_funding_rates(&info_client)
    )?;

    let mut token_vec: Vec<JointFundingRate> = vec![];

    for b_token in binance_tokens {
        if let Some(hl_token) = hyperliquid_tokens.iter().find(|t| t.name == b_token.name) {
            let token_comparison = JointFundingRate {
                name: b_token.name.clone(),
                hyperliquid_open_interest: hl_token.open_interest,
                binance_funding_rate: b_token.hourly_funding_rate,
                binance_mark_price: b_token.mark_price,
                hyperliquid_funding_rate: hl_token.hourly_funding_rate,
                funding_rate_difference: calculate_effective_rate(
                    b_token.hourly_funding_rate,
                    hl_token.hourly_funding_rate,
                ),
            };
            token_vec.push(token_comparison);
        };
    }

    token_vec.sort_by(|a, b| {
        b.funding_rate_difference
            .partial_cmp(&a.funding_rate_difference)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(token_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculate_effective_rate() {
        assert_eq!(calculate_effective_rate(0.001, -0.002), 0.003);
        assert_eq!(calculate_effective_rate(-0.001, 0.002), 0.003);
        assert_eq!(calculate_effective_rate(0.001, 0.002), 0.001);
        assert_eq!(calculate_effective_rate(-0.001, -0.002), 0.001);
    }

    #[tokio::test]
    async fn test_compare_funding_rates() {
        let fr = compare_funding_rates().await.unwrap();

        println!("{:#?}", fr.into_iter().take(5).collect::<Vec<_>>());
    }
}
