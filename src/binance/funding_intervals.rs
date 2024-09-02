use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceFundingInfo {
    pub symbol: String,
    pub funding_interval_hours: f64,
}

pub async fn retrieve_binance_funding_info(
    http_client: &Client,
) -> Result<Vec<BinanceFundingInfo>> {
    let req = http_client
        .get("https://fapi.binance.com/fapi/v1/fundingInfo")
        .send()
        .await
        .unwrap();

    let funding_info: Vec<BinanceFundingInfo> = req.json().await.unwrap();

    Ok(funding_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_funding_info() {
        let http_client = Client::new();
        let funding_rates = retrieve_binance_funding_info(&http_client).await.unwrap();

        println!("funding_info len: {}", funding_rates.len());
        println!("funding_info {:#?}", funding_rates);
    }
}
