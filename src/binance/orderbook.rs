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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_order_book() {
        let pair = "ETHUSDT".to_string();
        let result = retrieve_binance_order_book(pair, BidAsk::Ask).await;

        println!("{:#?}", result.unwrap());
    }
}
