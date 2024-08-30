mod funding_info;
mod funding_rate;
mod leverage;
mod open_interest;
mod parse_symbol;

use anyhow::Result;
use funding_info::retrieve_binance_funding_info;
use funding_rate::retrieve_binance_funding_rates;
use leverage::retrieve_binance_leverage;
use open_interest::retrieve_token_open_interest;
use reqwest::Client;
use std::collections::HashMap;

struct HashBucket {
    index_price: String,
    last_funding_rate: String,
    funding_interval_hours: u8,
    required_margin_percent: String,
    open_interest: String,
}

#[derive(Debug)]
struct BinanceToken {
    symbol: String,
    required_margin_percent: String,
    last_funding_rate: f64,
    funding_interval_hours: u8,
    open_interest: String,
}

pub async fn build_binance_tokens() -> Result<Vec<BinanceToken>> {
    let http_client = Client::new();

    let funding_rates = retrieve_binance_funding_rates(&http_client).await?;
    let funding_info = retrieve_binance_funding_info(&http_client).await?;
    let leverage_info = retrieve_binance_leverage(&http_client).await?;

    let mut token_map: HashMap<String, BinanceToken> = HashMap::new();

    for rate in funding_rates {
        let open_interest = retrieve_token_open_interest(&http_client, rate.symbol.clone()).await?;
        token_map.insert(
            rate.symbol.clone(),
            BinanceToken {
                symbol: rate.symbol.clone(),
                required_margin_percent: "".to_string(),
                last_funding_rate: rate.last_funding_rate.parse()?,
                open_interest: open_interest.open_interest,
                funding_interval_hours: 0,
            },
        );
    }

    for f_info in funding_info {
        if let Some(token) = token_map.get_mut(&f_info.symbol) {
            token.funding_interval_hours = f_info.funding_interval_hours;
        }
    }

    for li in leverage_info {
        if let Some(token) = token_map.get_mut(&li.symbol) {
            token.required_margin_percent = li.required_margin_percent;
        }
    }

    let binance_tokens: Vec<BinanceToken> = token_map.into_values().collect();

    Ok(binance_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_binance_tokens() {
        let b_tokens = build_binance_tokens().await.unwrap();

        println!("{:#?}", b_tokens);
    }
}
