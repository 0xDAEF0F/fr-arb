use crate::{binance::retrieve_binance_order_book, util::BidAsk};
use anyhow::{bail, Result};
use tokio::try_join;

pub async fn retrieve_token_price(token: String) -> Result<f64> {
    let (b_bids, b_asks) = try_join!(
        retrieve_binance_order_book(token.clone(), BidAsk::Bid),
        retrieve_binance_order_book(token.clone(), BidAsk::Ask),
    )?;

    if b_bids.limit_orders.is_empty() || b_asks.limit_orders.is_empty() {
        bail!("either bid/asks are empty")
    }

    let first_bid_price = b_bids.limit_orders[0].price;
    let first_ask_price = b_asks.limit_orders[0].price;

    // avg between the bid and ask
    Ok((first_bid_price + first_ask_price) / 2.0)
}
