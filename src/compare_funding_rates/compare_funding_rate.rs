use crate::binance::binance_funding_rate::retrieve_binance_hourly_funding_rates;
use crate::hyperliquid::hyperliquid_funding_rate::retrieve_hl_hourly_funding_rates;
use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;
use reqwest::Client;
use tokio::try_join;

#[derive(Debug)]
struct TokenComparison {
    name: String,
    binance_funding_rate: f64,
    hyperliquid_funding_rate: f64,
    funding_rate_difference: f64,
}

pub async fn compare_funding_rates() -> Result<Vec<TokenComparison>> {
    let http_client = Client::new();
    let info_client = InfoClient::new(None, None).await?;

    let (binance_tokens, hyperliquid_tokens) = try_join!(
        retrieve_binance_hourly_funding_rates(&http_client),
        retrieve_hl_hourly_funding_rates(&info_client)
    )?;

    let mut token_vec: Vec<TokenComparison> = vec![];

    for b_token in binance_tokens {
        if let Some(hl_token) = hyperliquid_tokens.iter().find(|t| t.name == b_token.name) {
            let token_comparison = TokenComparison {
                name: b_token.name.clone(),
                binance_funding_rate: b_token.hourly_funding_rate,
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

fn calculate_effective_rate(rate1: f64, rate2: f64) -> f64 {
    if rate1.signum() != rate2.signum() {
        rate1.abs() + rate2.abs()
    } else {
        rate1.abs().max(rate2.abs()) - rate1.abs().min(rate2.abs())
    }
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
