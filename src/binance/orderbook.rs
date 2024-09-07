use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

use crate::util::BidAsk;

#[derive(Debug, Deserialize)]
struct BinanceOrderBook {
    bids: Vec<Vec<String>>,
    asks: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceBidAsk {
    price: String,
    size: String,
}

async fn retrieve_binance_order_book(pair: String, ba: BidAsk) -> Result<Vec<BinanceBidAsk>> {
    let client = Client::new();

    let url = format!("https://fapi.binance.com/fapi/v1/depth?symbol={pair}");

    let response = client.get(&url).send().await?;
    let orderbook: BinanceOrderBook = response.json().await?;

    match ba {
        BidAsk::Ask => {
            let asks = orderbook
                .asks
                .into_iter()
                .map(|a| BinanceBidAsk {
                    price: a[0].to_string(),
                    size: a[1].to_string(),
                })
                .collect();
            return Ok(asks);
        }
        BidAsk::Bid => {
            let bids = orderbook
                .bids
                .into_iter()
                .map(|b| BinanceBidAsk {
                    price: b[0].to_string(),
                    size: b[1].to_string(),
                })
                .collect();
            return Ok(bids);
        }
    }
}

/// returns `None` if order book can't cover the amount
/// `amount` is USD
pub fn calculate_binance_execution_price(
    orderbook: Vec<BinanceBidAsk>,
    amount: f64,
) -> Option<f64> {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    if orderbook.is_empty() {
        return None;
    }

    for bid_ask in orderbook {
        let price: f64 = bid_ask.price.parse().unwrap();
        let size: f64 = bid_ask.size.parse().unwrap();
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
    async fn test_retrieve_binance_asks() {
        let pair = "ETHUSDT".to_string();
        let result = retrieve_binance_order_book(pair, BidAsk::Ask).await;

        println!("{:#?}", result.unwrap());
    }

    #[tokio::test]
    async fn test_average_price_for_purchase() {
        let result = retrieve_binance_order_book("TAOUSDT".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let avg_price = calculate_binance_execution_price(result, 50_000.0);

        println!("average execution price: {:?}", avg_price);
    }

    #[tokio::test]
    async fn test_average_price_for_sell() {
        let result = retrieve_binance_order_book("TAOUSDT".to_string(), BidAsk::Bid)
            .await
            .unwrap();

        let avg_price = calculate_binance_execution_price(result, 10_000.0);

        println!("average execution price: {:?}", avg_price);
    }

    #[tokio::test]
    async fn test_calculate_depth_orderbook() {
        let result = retrieve_binance_order_book("ETHUSDT".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let total_value_of_order_book: f64 = result.iter().fold(0.0, |acc, curr| {
            acc + (curr.price.parse::<f64>().unwrap() * curr.size.parse::<f64>().unwrap())
        });

        println!("Total value of orderbook: {}", total_value_of_order_book);
    }
}
