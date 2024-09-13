use crate::util::BidAsk;
use crate::util::LimitOrder;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct BinanceOrderBook {
    bids: Vec<Vec<String>>,
    asks: Vec<Vec<String>>,
}

pub async fn retrieve_binance_order_book(pair: String, ba: BidAsk) -> Result<Vec<LimitOrder>> {
    let client = Client::new();

    let url = format!("https://fapi.binance.com/fapi/v1/depth?symbol={pair}");

    let response = client.get(&url).send().await?;
    let orderbook: BinanceOrderBook = response.json().await?;

    match ba {
        BidAsk::Ask => {
            let asks = orderbook
                .asks
                .into_iter()
                .map(|a| {
                    Ok(LimitOrder {
                        price: a[0].parse::<f64>()?,
                        size: a[1].parse::<f64>()?,
                    })
                })
                .collect::<Result<Vec<LimitOrder>>>()?;
            Ok(asks)
        }
        BidAsk::Bid => {
            let bids = orderbook
                .bids
                .into_iter()
                .map(|b| {
                    Ok(LimitOrder {
                        price: b[0].parse::<f64>()?,
                        size: b[1].parse::<f64>()?,
                    })
                })
                .collect::<Result<Vec<LimitOrder>>>()?;
            Ok(bids)
        }
    }
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
    async fn test_calculate_depth_orderbook() {
        let result = retrieve_binance_order_book("ETHUSDT".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let total_value_of_order_book: f64 = result
            .iter()
            .fold(0.0, |acc, curr| acc + (curr.price * curr.size));

        println!("Total value of orderbook: {}", total_value_of_order_book);
    }
}
