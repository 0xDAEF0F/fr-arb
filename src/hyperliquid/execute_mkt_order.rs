use crate::{
    constants::EXECUTION_SLIPPAGE,
    util::{OrderFilled, Platform, Side},
};
use anyhow::{bail, Result};
use ethers::signers::{coins_bip39::English, LocalWallet, MnemonicBuilder};
use hyperliquid_rust_sdk::{
    BaseUrl, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, MarketOrderParams,
};

use super::get_wallet;

pub async fn execute_mkt_order(token: String, size: f64, is_buy: bool) -> Result<OrderFilled> {
    let hl_client = setup_hl_client().await?;

    let market_open_params = MarketOrderParams {
        asset: token.as_str(),
        is_buy,
        sz: size,
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
            return Ok(OrderFilled {
                token,
                platform: Platform::Hyperliquid,
                size: order.total_sz.parse().unwrap(),
                avg_price: order.avg_px.parse().unwrap(),
                side: if is_buy { Side::Buy } else { Side::Sell },
            });
        }
    }

    bail!("could not execute market order")
}

async fn setup_hl_client() -> Result<ExchangeClient> {
    let wallet = get_wallet()?;

    let exchange_client =
        ExchangeClient::new(None, wallet, Some(BaseUrl::Mainnet), None, None).await?;

    Ok(exchange_client)
}
