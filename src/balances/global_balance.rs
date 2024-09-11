use crate::binance::account_information::retrieve_binance_account_info;
use crate::hyperliquid::account_information::retrieve_hl_account_info;
use anyhow::Result;
use prettytable::{Cell, Row, Table};

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

    let b_balance = (balances.binance.parse::<f64>()? * 100.0).round() / 100.0;
    let b_bal = format!("$ {b_balance}");

    let hl_balance = (balances.hyperliquid.parse::<f64>()? * 100.0).round() / 100.0;
    let hl_bal = format!("$ {hl_balance}");

    let total_bal = format!("$ {}", b_balance + hl_balance);

    table.add_row(Row::new(vec![Cell::new("Balances")]));
    table.add_row(Row::new(vec![
        Cell::new("Binance"),
        Cell::new(b_bal.as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Hyperliquid"),
        Cell::new(hl_bal.as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Total"),
        Cell::new(total_bal.as_str()),
    ]));

    Ok(table.to_string())
}
