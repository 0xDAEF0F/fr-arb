use crate::binance::{
    account_information::retrieve_binance_account_info,
    binance_funding_rate::retrieve_binance_hourly_funding_rates,
};
use crate::hyperliquid::account_information::retrieve_hl_account_info;
use anyhow::Result;
use prettytable::{Cell, Row, Table};
use reqwest::Client;

pub struct Balance {
    pub binance: String,
    pub hyperliquid: String,
}

pub struct Position {
    pub coin: String,      // without quote
    pub direction: String, // short || long
    pub size: f64,         // amount of tokens/cryptocurrency
    pub entry_price: f64,
    pub pnl: f64,
    pub funding_rate: f64, // annualized
}

async fn retrieve_account_balance() -> Result<Balance> {
    let binance_acct_info = retrieve_binance_account_info().await?;
    let hyperliquid_acct_info = retrieve_hl_account_info().await?;

    let balance = Balance {
        binance: binance_acct_info.total_margin_balance,
        hyperliquid: hyperliquid_acct_info.cross_margin_summary.account_value,
    };

    Ok(balance)
}

async fn retrieve_account_open_position() -> Result<Vec<Position>> {
    let client = Client::new();
    let binance_acct_info = retrieve_binance_account_info().await?;
    let hyperliquid_acct_info = retrieve_hl_account_info().await?;
    let funding_rates = retrieve_binance_hourly_funding_rates(&client).await?;

    let binance_positions: Vec<_> = binance_acct_info
        .positions
        .into_iter()
        .map(|p| {
            let coin = p.symbol.trim_end_matches("USDT").to_string();
            let pnl: f64 = p.unrealized_profit.parse().unwrap();
            let funding_rate = funding_rates
                .iter()
                .find(|&rate| rate.name == coin)
                .map(|rate| ((rate.hourly_funding_rate * 24.0 * 365.0) * 100.0).round() / 100.0)
                .expect("funding rate not found");
            let direction = p.position_side.to_lowercase();
            let size = p.position_amt.parse::<f64>().unwrap().abs();
            let entry_price = (p.notional.parse::<f64>().unwrap().abs()
                + p.unrealized_profit.parse::<f64>().unwrap())
                / size;
            Position {
                coin,
                direction,
                entry_price,
                funding_rate,
                pnl,
                size,
            }
        })
        .collect();

    todo!()
}

pub async fn build_account_balance_table() -> Result<String> {
    let balances = retrieve_account_balance().await?;
    let total_balance = balances.binance.parse::<f64>()? + balances.hyperliquid.parse::<f64>()?;

    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new("Balances")]));
    table.add_row(Row::new(vec![
        Cell::new("Binance"),
        Cell::new(balances.binance.as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Hyperliquid"),
        Cell::new(balances.hyperliquid.as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Total"),
        Cell::new(&total_balance.to_string()),
    ]));

    Ok(table.to_string())
}
