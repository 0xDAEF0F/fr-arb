use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ResBinanceOrderBook {
    bids: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceBid {
    price: f64,
    size: f64,
}

async fn retrieve_binance_bids(pair: String) -> Result<Vec<BinanceBid>> {
    let client = Client::new();

    let url = format!("https://fapi.binance.com/fapi/v1/depth?symbol={pair}");

    let response = client.get(&url).send().await?;
    let orderbook: ResBinanceOrderBook = response.json().await?;

    let bids: Vec<BinanceBid> = orderbook
        .bids
        .into_iter()
        .map(|bid| {
            let price = bid[0].parse::<f64>().unwrap();
            let size = bid[1].parse::<f64>().unwrap();
            BinanceBid { price, size }
        })
        .collect();

    Ok(bids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_bids() {
        let pair = "BTCUSDT".to_string();
        let result = retrieve_binance_bids(pair).await;

        println!("{:#?}", result.unwrap());
    }
}
