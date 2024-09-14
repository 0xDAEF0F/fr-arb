use super::retrieve_hl_order_book;
use crate::{constants::EXECUTION_SLIPPAGE, quote::retrieve_quote, util::BidAsk};
use anyhow::{bail, Result};
use ethers::signers::{coins_bip39::English, LocalWallet, MnemonicBuilder};
use hyperliquid_rust_sdk::{
    BaseUrl, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, FilledOrder,
    MarketOrderParams,
};

async fn execute_mkt_order(token: String, amt: f64, is_buy: bool) -> Result<FilledOrder> {
    let hl_client = setup_hl_client().await?;

    let ba = if is_buy { BidAsk::Ask } else { BidAsk::Bid };
    let hl_orderbook = retrieve_hl_order_book(token.clone(), ba).await?;

    let quote = retrieve_quote(hl_orderbook, amt)?;

    let market_open_params = MarketOrderParams {
        asset: token.as_str(),
        is_buy: false,
        sz: quote.size,
        px: None,
        slippage: Some(EXECUTION_SLIPPAGE),
        cloid: None,
        wallet: None,
    };

    if let ExchangeResponseStatus::Ok(exchange_response) =
        hl_client.market_open(market_open_params).await?
    {
        if let ExchangeDataStatus::Filled(order) =
            exchange_response.data.unwrap().statuses[0].clone()
        {
            return Ok(order);
        }
    }

    bail!("could not execute market order")
}

async fn setup_hl_client() -> Result<ExchangeClient> {
    let mnemonic = std::env::var("MNEMONIC")?;

    let wallet: LocalWallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic.as_str())
        .build()?;

    let exchange_client =
        ExchangeClient::new(None, wallet, Some(BaseUrl::Mainnet), None, None).await?;

    Ok(exchange_client)
}
