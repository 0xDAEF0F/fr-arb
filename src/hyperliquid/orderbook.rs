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
/// `amount` is USD
pub fn calculate_hl_execution_price(orderbook: Vec<HlBidAsk>, amount: f64) -> Option<f64> {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    if orderbook.is_empty() {
        return None;
    }

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
            remaining_amount -= quantity_needed * price;

            break;
        }

        total_cost += total;
        total_quantity += size;
        remaining_amount -= total;
    }

    // order book does not have enough orders for the amount
    if remaining_amount > 0.0 {
        return None;
    }

    Some(total_cost / total_quantity)
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

    #[tokio::test]
    async fn test_average_price_for_purchase() {
        let result = retrieve_hl_order_book("TAO".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let avg_price = calculate_hl_execution_price(result, 10_000.0);

        println!("average execution price: {:?}", avg_price);
    }

    #[tokio::test]
    async fn test_calculate_depth_orderbook() {
        let result = retrieve_hl_order_book("ETH".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let total_value_of_order_book: f64 = result.iter().fold(0.0, |acc, curr| {
            acc + (curr.px.parse::<f64>().unwrap() * curr.sz.parse::<f64>().unwrap())
        });

        println!("Total value of orderbook: {}", total_value_of_order_book);
    }
}
