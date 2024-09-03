use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ResHyperliquidOrderBook {
    levels: Vec<Vec<HlBidAsk>>,
}

#[derive(Debug, Deserialize)]
struct HlBidAsk {
    px: String,
    sz: String,
}

async fn retrieve_hyperliquid_bids(pair: String) -> Result<Vec<HlBidAsk>> {
    let client = Client::new();

    let url = "https://api.hyperliquid.xyz/info";
    let body = serde_json::json!({
        "type": "l2Book",
        "coin": format!("{pair}")
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let orderbook: ResHyperliquidOrderBook = response.json().await?;

    let mut bids = Vec::new();
    if let Some(first_level) = orderbook.levels.into_iter().next() {
        bids = first_level;
    }

    Ok(bids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_hyperliquid_bids() {
        let pair = "ETH".to_string();
        let result = retrieve_hyperliquid_bids(pair).await.unwrap();

        println!("{:#?}", result);
    }
}
