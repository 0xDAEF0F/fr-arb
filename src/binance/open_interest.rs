use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterest {
    pub symbol: String,
    pub open_interest: String,
}

pub async fn retrieve_token_open_interest(token: String) -> Result<OpenInterest> {
    let client = Client::new();

    let url = format!("https://fapi.binance.com/fapi/v1/openInterest?symbol={token}USDT");
    let req = client.get(url).send().await?;

    let pair_oi: OpenInterest = req.json().await?;

    Ok(pair_oi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_leverage() {
        let pair_oi = retrieve_token_open_interest("BTC".to_string())
            .await
            .unwrap();

        println!("{:#?}", pair_oi);
    }
}
