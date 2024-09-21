use crate::{binance::retrieve_binance_order_book, util::Orderbook};
use anyhow::{bail, Result};

pub async fn retrieve_token_price(token: String) -> Result<f64> {
    let b_orderbook = retrieve_binance_order_book(token.clone()).await?;

    if b_orderbook.bids.is_empty() || b_orderbook.asks.is_empty() {
        bail!("Either bid/asks are empty")
    }

    let first_bid_price = b_orderbook.bids[0].price;
    let first_ask_price = b_orderbook.asks[0].price;

    // avg between the bid and ask
    Ok((first_bid_price + first_ask_price) / 2.0)
}

pub fn get_mid_price(orderbook: &Orderbook) -> Result<f64> {
    if orderbook.asks.is_empty() || orderbook.bids.is_empty() {
        bail!("Bids or asks are empty")
    }

    Ok((orderbook.bids[0].price + orderbook.asks[0].price) / 2.0)
}
