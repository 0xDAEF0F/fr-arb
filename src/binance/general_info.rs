use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    symbols: Vec<TokenLeverage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenLeverage {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub required_margin_percent: f64,
    #[serde(deserialize_with = "deserialize_nth_item")]
    pub filters: MarketLotSize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketLotSize {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub step_size: f64,
}

pub async fn retrieve_binance_general_info() -> Result<Vec<TokenLeverage>> {
    let client = Client::new();

    let req = client
        .get("https://fapi.binance.com/fapi/v1/exchangeInfo")
        .send()
        .await?;

    let response: Response = req.json().await?;

    let tokens = response.symbols;

    Ok(tokens)
}

fn deserialize_nth_item<'de, D>(deserializer: D) -> Result<MarketLotSize, D::Error>
where
    D: Deserializer<'de>,
{
    let values: Vec<Value> = Deserialize::deserialize(deserializer)?;

    let n = 2;

    let nth_value = values
        .get(n)
        .ok_or_else(|| serde::de::Error::custom(format!("No item at index {}", n)))?;

    MarketLotSize::deserialize(nth_value).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_leverage() {
        let tokens = retrieve_binance_general_info().await.unwrap();

        println!("{:#?}", tokens.into_iter().take(5).collect::<Vec<_>>())
    }
}
