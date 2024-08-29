use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenInterest {
    symbol: String,
    open_interest: String,
}

async fn retrieve_token_open_interest(http_client: &Client, pair: String) -> Result<OpenInterest> {
    let url = format!("https://fapi.binance.com/fapi/v1/openInterest?symbol={pair}");
    let req = http_client.get(url).send().await?;

    let pair_oi: OpenInterest = req.json().await?;

    Ok(pair_oi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_leverage() {
        let http_client = Client::new();

        let pair_oi = retrieve_token_open_interest(&http_client, "BTCUSDT".to_string())
            .await
            .unwrap();

        println!("{:#?}", pair_oi);
    }
}
