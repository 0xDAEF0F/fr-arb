use crate::util::{BidAsk, LimitOrder};
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

pub async fn retrieve_hl_order_book(pair: String, ba: BidAsk) -> Result<Vec<LimitOrder>> {
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

    let ba = match ba {
        BidAsk::Bid => orderbook.levels.into_iter().nth(0).unwrap(),
        BidAsk::Ask => orderbook.levels.into_iter().nth(1).unwrap(),
    };

    ba.into_iter()
        .map(|ba| -> Result<LimitOrder> {
            Ok(LimitOrder {
                price: ba.px.parse()?,
                size: ba.sz.parse()?,
            })
        })
        .collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_hyperliquid_asks() {
        let result = retrieve_hl_order_book("ETH".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        println!("{:#?}", result);
    }
}
