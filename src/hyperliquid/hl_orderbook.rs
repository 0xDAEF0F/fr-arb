use crate::util::{LimitOrder, Orderbook, Platform};
use anyhow::{bail, Result};
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

pub async fn retrieve_hl_order_book(token: &str) -> Result<Orderbook> {
    let client = Client::new();

    let url = "https://api.hyperliquid.xyz/info";
    let body = serde_json::json!({
        "type": "l2Book",
        "coin": format!("{token}")
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("Could not retrieve orderbook for {} in Hyperliquid.", token);
    }

    let orderbook: ResHyperliquidOrderBook = response.json().await?;

    let (bids, asks) = match orderbook.levels.as_slice() {
        [bids, asks, ..] => (bids, asks),
        _ => bail!("Invalid orderbook structure"),
    };

    let bids = parse_orders(bids)?;
    let asks = parse_orders(asks)?;

    Ok(Orderbook {
        platform: Platform::Hyperliquid,
        bids,
        asks,
    })
}

fn parse_orders(orders: &[HlBidAsk]) -> Result<Vec<LimitOrder>> {
    orders
        .iter()
        .map(|ba| -> Result<LimitOrder> {
            Ok(LimitOrder {
                price: ba.px.parse()?,
                size: ba.sz.parse()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_hyperliquid_asks() {
        let result = retrieve_hl_order_book("kPEPE").await.unwrap();

        println!("{:#?}", result);

        assert_eq!(result.platform, Platform::Hyperliquid);
    }
}
