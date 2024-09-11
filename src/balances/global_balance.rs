use crate::binance::account_information::retrieve_binance_account_info;
use crate::hyperliquid::account_information::retrieve_hl_account_info;
use anyhow::Result;
use numfmt::{Formatter, Precision::Decimals};
use prettytable::{Cell, Row, Table};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Balance {
    pub binance: String,
    pub hyperliquid: String,
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

pub async fn build_account_balance_table() -> Result<String> {
    let balances = retrieve_account_balance().await?;

    let mut table = Table::new();

    let mut f = Formatter::new()
        .separator(',')?
        .prefix("$")?
        .precision(Decimals(2));

    let b_balance = balances.binance.parse::<f64>()?;
    let hl_balance = balances.hyperliquid.parse::<f64>()?;

    table.add_row(Row::new(vec![Cell::new("Balances")]));
    table.add_row(Row::new(vec![
        Cell::new("Binance"),
        Cell::new(f.fmt2(b_balance)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Hyperliquid"),
        Cell::new(f.fmt2(hl_balance)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Total"),
        Cell::new(f.fmt2(b_balance + hl_balance)), // total balance
    ]));

    Ok(table.to_string())
}
