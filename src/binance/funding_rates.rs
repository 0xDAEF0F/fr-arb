use super::leverage::retrieve_binance_leverage;
use crate::binance::{
    funding_intervals::retrieve_binance_funding_info,
    raw_funding_rate::retrieve_binance_raw_funding_rates,
};
use anyhow::Result;
use reqwest::Client;

#[derive(Debug)]
pub struct BinanceFundingRate {
    pub name: String,
    pub hourly_funding_rate: f64,
    pub max_leverage: f64,
    pub mark_price: f64,
}

pub async fn retrieve_binance_hourly_funding_rates(
    http_client: &Client,
) -> Result<Vec<BinanceFundingRate>> {
    let raw_funding_rates = retrieve_binance_raw_funding_rates(&http_client).await?;
    let funding_info = retrieve_binance_funding_info(&http_client).await?;
    let token_leverage = retrieve_binance_leverage(&http_client).await?;

    let mut hourly_funding_rates = Vec::new();

    for rates in raw_funding_rates {
        let interval = funding_info
            .iter()
            .find(|info| info.symbol == rates.symbol)
            .map(|info| info.funding_interval_hours)
            .unwrap_or(8.0);
        let required_margin_percent = token_leverage
            .iter()
            .find(|t| t.symbol == rates.symbol)
            .map(|t| t.required_margin_percent)
            .unwrap_or(5.0);

        let hourly_rate = rates.last_funding_rate / interval;
        let max_leverage = 100.0 / required_margin_percent;

        hourly_funding_rates.push(BinanceFundingRate {
            name: rates.symbol,
            mark_price: rates.mark_price,
            hourly_funding_rate: hourly_rate,
            max_leverage,
        });
    }

    // Remove anything other than USDT and only use Base symbol name
    let hourly_funding_rates = hourly_funding_rates
        .into_iter()
        .filter(|t| t.name.ends_with("USDT"))
        .map(|t| BinanceFundingRate {
            name: t.name.trim_end_matches("USDT").to_string(),
            mark_price: t.mark_price,
            hourly_funding_rate: t.hourly_funding_rate,
            max_leverage: t.max_leverage,
        })
        .collect();

    Ok(hourly_funding_rates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_binance_hourly_funding_rates() {
        let client = Client::new();
        let hourly_funding_rates = retrieve_binance_hourly_funding_rates(&client)
            .await
            .unwrap();
        // println!(
        //     "{:#?}",
        //     hourly_funding_rates.iter().find(|h| h.name == "ZRO")
        // );
        // println!(
        //     "hourly_funding_rates len: {:#?}",
        //     hourly_funding_rates.len()
        // );
        println!("{:#?}", hourly_funding_rates);
    }
}
