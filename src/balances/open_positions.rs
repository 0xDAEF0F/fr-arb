use crate::{
    binance::{
        account_information::retrieve_binance_account_info,
        binance_funding_rate::retrieve_binance_hourly_funding_rates,
    },
    hyperliquid::{
        account_information::retrieve_hl_account_info,
        hyperliquid_funding_rate::retrieve_hl_hourly_funding_rates,
    },
};
use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;
use prettytable::{Cell, Row, Table};
use reqwest::Client;

#[derive(Debug)]
pub struct Position {
    pub platform: String,
    pub coin: String,      // without quote
    pub direction: String, // short || long
    pub size: f64,         // amount of tokens/cryptocurrency
    pub entry_price: f64,
    pub pnl: f64,
    pub funding_rate: f64, // annualized
}

async fn retrieve_account_open_positions() -> Result<Vec<Position>> {
    let client = Client::new();
    let info_client = InfoClient::new(None, None).await.unwrap();

    let binance_acct_info = retrieve_binance_account_info().await?;
    let hyperliquid_acct_info = retrieve_hl_account_info().await?;

    let binance_funding_rates = retrieve_binance_hourly_funding_rates(&client).await?;
    let hl_funding_rates = retrieve_hl_hourly_funding_rates(&info_client).await?;

    let mut binance_positions: Vec<_> = binance_acct_info
        .positions
        .into_iter()
        .map(|p| {
            let coin = p.symbol.trim_end_matches("USDT").to_string();
            let pnl: f64 = p.unrealized_profit.parse().unwrap();
            let funding_rate = binance_funding_rates
                .iter()
                .find(|&rate| rate.name == coin)
                .map(|rate| (rate.hourly_funding_rate * 24.0 * 365.0))
                .expect("funding rate not found");
            let direction = p.position_side.to_lowercase();
            let size = p.position_amt.parse::<f64>().unwrap().abs();
            let entry_price = (p.notional.parse::<f64>().unwrap().abs()
                + p.unrealized_profit.parse::<f64>().unwrap())
                / size;
            Position {
                platform: "binance".to_string(),
                coin,
                direction,
                entry_price,
                funding_rate,
                pnl,
                size,
            }
        })
        .collect();

    let hyperliquid_positions: Vec<_> = hyperliquid_acct_info
        .asset_positions
        .into_iter()
        .map(|p| {
            let coin = p.position.coin;
            let pnl: f64 = p.position.unrealized_pnl.parse().unwrap();
            let funding_rate = hl_funding_rates
                .iter()
                .find(|&rate| rate.name == coin)
                .map(|rate| rate.hourly_funding_rate * 24.0 * 365.0)
                .expect("funding rate not found (hl)");
            let direction = if p.position.szi.parse::<f64>().unwrap().is_sign_positive() {
                "long".to_string()
            } else {
                "short".to_string()
            };
            let size = p.position.szi.parse::<f64>().unwrap().abs();
            let entry_price = p.position.entry_px.parse::<f64>().unwrap();
            Position {
                platform: "hyperliquid".to_string(),
                coin,
                direction,
                entry_price,
                funding_rate,
                pnl,
                size,
            }
        })
        .collect();

    binance_positions.extend(hyperliquid_positions);
    binance_positions.sort_by(|a, b| a.coin.cmp(&b.coin));

    let total_positions = binance_positions;

    Ok(total_positions)
}

pub async fn build_account_open_positions_table() -> Result<String> {
    let open_positions = retrieve_account_open_positions().await?;

    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new("Open Positions")]));
    table.add_row(Row::new(vec![
        Cell::new("Platform"),
        Cell::new("Coin"),
        Cell::new("Direction"),
        Cell::new("size (amt tokens)"),
        Cell::new("entry price"),
        Cell::new("pnl"),
        Cell::new("funding rate (annualized)"),
    ]));

    for position in open_positions {
        table.add_row(Row::new(vec![
            Cell::new(position.platform.as_str()),
            Cell::new(position.coin.as_str()),
            Cell::new(position.direction.as_str()),
            Cell::new(position.size.to_string().as_str()),
            Cell::new(position.entry_price.to_string().as_str()),
            Cell::new(position.pnl.to_string().as_str()),
            Cell::new(position.funding_rate.to_string().as_str()),
        ]));
    }

    Ok(table.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_account_open_positions() -> Result<()> {
        dotenv::dotenv().ok();

        let positions = retrieve_account_open_positions().await?;

        println!("{positions:#?}");

        Ok(())
    }
}
