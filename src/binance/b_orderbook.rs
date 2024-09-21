use crate::util::LimitOrder;
use crate::util::Orderbook;
use crate::util::Platform;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct BinanceOrderBook {
    bids: Vec<Vec<String>>,
    asks: Vec<Vec<String>>,
}

pub async fn retrieve_binance_order_book(token: &str) -> Result<Orderbook> {
    let client = Client::new();

    let url = format!("https://fapi.binance.com/fapi/v1/depth?symbol={token}USDT");

    let response = client.get(&url).send().await?;
    let orderbook: BinanceOrderBook = response.json().await?;

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

    Ok(Orderbook {
        platform: Platform::Binance,
        bids,
        asks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_asks() {
        let result = retrieve_binance_order_book("ETH").await.unwrap();

        println!("{:#?}", result);

        assert_eq!(result.platform, Platform::Binance);
    }
}
