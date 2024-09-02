use crate::{
    binance::build_binance_tokens, hyperliquid::hyperliquid_funding_rate::retrieve_hl_tokens,
};
use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;
use std::collections::HashMap;

#[derive(Debug)]
struct TokenComparison {
    name: String,
    binance_funding_rate: Option<f64>,
    hyperliquid_funding_rate: Option<f64>,
    funding_rate_difference: Option<f64>,
}

pub async fn compare_funding_rates() -> Result<Vec<TokenComparison>> {
    let binance_tokens = build_binance_tokens().await?;

    let info_client = InfoClient::new(None, None).await?;
    let hyperliquid_tokens = retrieve_hl_tokens(&info_client).await;

    let mut token_map: HashMap<String, TokenComparison> = HashMap::new();

    for token in binance_tokens {
        token_map.insert(
            token.name.clone(),
            TokenComparison {
                name: token.name,
                binance_funding_rate: Some(token.hourly_funding_rate),
                hyperliquid_funding_rate: None,
                funding_rate_difference: None,
            },
        );
    }

    for token in hyperliquid_tokens {
        if let Some(comparison) = token_map.get_mut(&token.name) {
            comparison.hyperliquid_funding_rate = Some(token.hourly_funding_rate);
            comparison.funding_rate_difference = Some(calculate_effective_rate(
                comparison.binance_funding_rate.unwrap(),
                token.hourly_funding_rate,
            ));
        } else {
            token_map.insert(
                token.name.clone(),
                TokenComparison {
                    name: token.name,
                    binance_funding_rate: None,
                    hyperliquid_funding_rate: Some(token.hourly_funding_rate),
                    funding_rate_difference: None,
                },
            );
        }
    }

    let comparisons: Vec<TokenComparison> = token_map.into_values().collect();
    let mut comparisons: Vec<TokenComparison> = comparisons
        .into_iter()
        .filter(|t| t.funding_rate_difference.is_some())
        .collect();

    // Sort comparisons by funding_rate_difference in descending order
    comparisons.sort_by(|a, b| {
        b.funding_rate_difference
            .unwrap_or(0.0)
            .partial_cmp(&a.funding_rate_difference.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(comparisons)
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
