use crate::util::BidAsk;
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

async fn retrieve_hl_order_book(pair: String, ba: BidAsk) -> Result<Vec<HlBidAsk>> {
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

    match ba {
        BidAsk::Bid => Ok(orderbook.levels.into_iter().nth(0).unwrap()),
        BidAsk::Ask => Ok(orderbook.levels.into_iter().nth(1).unwrap()),
    }
}

/// returns `None` if order book can't cover the amount
fn calculate_execution_price(orderbook: Vec<HlBidAsk>, amount: f64) -> f64 {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    for bid_ask in orderbook {
        let price: f64 = bid_ask.px.parse().unwrap();
        let size: f64 = bid_ask.sz.parse().unwrap();
        let total = price * size;

        // check if total is greater that remaining amount
        if total > remaining_amount {
            // partially take from that order
            let quantity_needed = remaining_amount / price;

            total_cost += quantity_needed * price;
            total_quantity += quantity_needed;

            break;
        }

        total_cost += total;
        total_quantity += size;
        remaining_amount -= total;
    }

    total_cost / total_quantity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_hyperliquid_bids() {
        let pair = "ETH".to_string();

        let result = retrieve_hl_order_book(pair, BidAsk::Ask).await.unwrap();

        println!("{:#?}", result);

        let resultt = calculate_execution_price(result, 100.0);

        println!("{:#?}", resultt);
    }
}
