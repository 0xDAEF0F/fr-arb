use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

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
}

pub async fn retrieve_binance_leverage(http_client: &Client) -> Result<Vec<TokenLeverage>> {
    let req = http_client
        .get("https://fapi.binance.com/fapi/v1/exchangeInfo")
        .send()
        .await?;

    let response: Response = req.json().await?;

    let tokens = response.symbols;

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_leverage() {
        let http_client = Client::new();
        let tokens = retrieve_binance_leverage(&http_client).await.unwrap();
        println!("{:#?}", tokens.into_iter().take(5).collect::<Vec<_>>())
    }
}
