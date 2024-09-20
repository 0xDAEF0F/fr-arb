use crate::{
    binance::retrieve_binance_order_book, hyperliquid::retrieve_hl_order_book, util::Orderbook,
};
use anyhow::{bail, Result};
use tokio::try_join;

/// first element in the tuple is the binance orderbook
pub async fn retrieve_orderbooks(token: String) -> Result<(Orderbook, Orderbook)> {
    let maybe_orderbooks = try_join!(
        retrieve_binance_order_book(token.clone()),
        retrieve_hl_order_book(token)
    );

    match maybe_orderbooks {
        Err(e) => bail!("Could not retrieve orderbooks. {}", e),
        Ok(orderbooks) => Ok(orderbooks),
    }
}
