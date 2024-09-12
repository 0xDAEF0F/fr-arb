use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceIndexFundingRate {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub last_funding_rate: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub mark_price: f64,
}

pub async fn retrieve_binance_raw_funding_rates(
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
    async fn test_funding_rates() {
        let http_client = Client::new();
        let funding_rates = retrieve_binance_raw_funding_rates(&http_client)
            .await
            .unwrap();

        println!("funding_rates len: {}", funding_rates.len());
        println!("funding_rates {:#?}", funding_rates);
    }
}
