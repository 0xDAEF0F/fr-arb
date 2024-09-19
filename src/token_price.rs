use crate::binance::retrieve_binance_order_book;
use anyhow::{bail, Result};

pub async fn retrieve_token_price(token: String) -> Result<f64> {
    let b_orderbook = retrieve_binance_order_book(token.clone()).await?;

    if b_orderbook.bids.is_empty() || b_orderbook.asks.is_empty() {
        bail!("either bid/asks are empty")
    }

    let first_bid_price = b_orderbook.bids[0].price;
    let first_ask_price = b_orderbook.asks[0].price;

    // avg between the bid and ask
    Ok((first_bid_price + first_ask_price) / 2.0)
}
