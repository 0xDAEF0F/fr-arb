mod binance_funding_rate;
mod funding_intervals;
mod leverage;
mod open_interest;
mod parse_symbol;
mod raw_funding_rate;

use anyhow::Result;
use funding_intervals::retrieve_binance_funding_info;
use leverage::retrieve_binance_leverage;
use raw_funding_rate::retrieve_binance_raw_funding_rates;
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug)]
struct RawBinanceToken {
    symbol: String,
    required_margin_percent: f64,
    last_funding_rate: f64,
    funding_interval_hours: f64,
}

#[derive(Debug)]
pub struct BinanceToken {
    pub name: String,
    pub max_leverage: u8,
    pub hourly_funding_rate: f64,
}

pub async fn build_binance_raw_tokens() -> Result<Vec<RawBinanceToken>> {
    let http_client = Client::new();

    let funding_rates = retrieve_binance_raw_funding_rates(&http_client).await?;
    let funding_info = retrieve_binance_funding_info(&http_client).await?;
    let leverage_info = retrieve_binance_leverage(&http_client).await?;

    let mut token_map: HashMap<String, RawBinanceToken> = HashMap::new();

    for rate in funding_rates {
        token_map.insert(
            rate.symbol.clone(),
            RawBinanceToken {
                symbol: rate.symbol.clone(),
                required_margin_percent: 0.0,
                last_funding_rate: rate.last_funding_rate,
                funding_interval_hours: 0.0,
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

    let binance_tokens: Vec<RawBinanceToken> = token_map.into_values().collect();

    // Remove symbols that don't end with USDT
    // all `funding_interval_hours` that weren't filled, are going to be 8 by default
    let binance_tokens: Vec<RawBinanceToken> = binance_tokens
        .into_iter()
        .filter(|t| t.symbol.ends_with("USDT"))
        .map(|mut t| {
            if t.funding_interval_hours == 0.0 {
                t.funding_interval_hours = 8.0;
            }
            t
        })
        .collect();

    Ok(binance_tokens)
}

pub async fn build_binance_tokens() -> Result<Vec<BinanceToken>> {
    let raw_binance_tokens = build_binance_raw_tokens().await?;

    let tokens = raw_binance_tokens
        .into_iter()
        .map(|token| {
            let pair = parse_symbol::parse_symbol(token.symbol).unwrap();

            let hourly_funding_rate = token.last_funding_rate / token.funding_interval_hours as f64;
            let max_leverage = (100_f64 / token.required_margin_percent) as u8;

            BinanceToken {
                name: pair.base,
                max_leverage,
                hourly_funding_rate,
            }
        })
        .collect();

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_binance_tokens() {
        let tokens = build_binance_tokens().await.unwrap();
        println!("{:#?}", tokens);
    }
}
