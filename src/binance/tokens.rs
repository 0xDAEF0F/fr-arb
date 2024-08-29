use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceIndexFundingRate {
    symbol: String,
    index_price: String,
    last_funding_rate: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceFundingInfo {
    symbol: String,
    funding_interval_hours: u8,
}

async fn retrieve_binance_tokens() {
    let http_client = Client::new();

    let req = http_client
        .get("https://fapi.binance.com/fapi/v1/fundingInfo")
        .send()
        .await
        .unwrap();

    let funding_info: Vec<BinanceFundingInfo> = req.json().await.unwrap();

    println!("{:#?}", funding_info);

    let req = http_client
        .get("https://fapi.binance.com/fapi/v1/premiumIndex")
        .send()
        .await
        .unwrap();

    let mark_funding_rate: Vec<BinanceIndexFundingRate> = req.json().await.unwrap();

    println!("{:#?}", mark_funding_rate);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_tokens() {
        retrieve_binance_tokens().await;
    }
}
