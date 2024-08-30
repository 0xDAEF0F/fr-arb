use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceFundingInfo {
    pub symbol: String,
    pub funding_interval_hours: u8,
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
