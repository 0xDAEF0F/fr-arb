use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceIndexFundingRate {
    pub symbol: String,
    pub index_price: String,
    pub last_funding_rate: String,
}

pub async fn retrieve_binance_funding_rates(
    http_client: &Client,
) -> Result<Vec<BinanceIndexFundingRate>> {
    let req = http_client
        .get("https://fapi.binance.com/fapi/v1/premiumIndex")
        .send()
        .await?;

    let funding_rates: Vec<BinanceIndexFundingRate> = req.json().await?;

    Ok(funding_rates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_tokens() {
        let http_client = Client::new();
        let funding_rates = retrieve_binance_funding_rates(&http_client).await.unwrap();
        assert!(!funding_rates.is_empty());
    }
}
