use crate::binance::account_information::retrieve_binance_account_info;
use crate::hyperliquid::account_information::retrieve_hl_account_info;
use anyhow::Result;
use numfmt::{Formatter, Precision::Decimals};
use prettytable::{Cell, Row, Table};

#[derive(Debug)]
pub struct Balance {
    pub binance_balance: f64,
    pub binance_open_positions: f64,
    pub hyperliquid_balance: f64,
    pub hyperliquid_open_positions: f64,
}

async fn retrieve_account_balance() -> Result<Balance> {
    let binance_acct_info = retrieve_binance_account_info().await?;
    let hyperliquid_acct_info = retrieve_hl_account_info().await?;

    let binance_open_positions: f64 = binance_acct_info.positions.iter().map(|p| p.notional).sum();

    let balance = Balance {
        binance_balance: binance_acct_info.total_margin_balance,
        binance_open_positions,
        hyperliquid_balance: hyperliquid_acct_info.cross_margin_summary.account_value,
        hyperliquid_open_positions: hyperliquid_acct_info.cross_margin_summary.total_ntl_pos,
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

    let b_balance = balances.binance_balance;
    let hl_balance = balances.hyperliquid_balance;

    table.add_row(Row::new(vec![
        Cell::new("Balances"),
        Cell::new("Amount"),
        Cell::new("Open Positions"),
        Cell::new("Leverage"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Binance"),
        Cell::new(f.fmt2(b_balance)),
        Cell::new(f.fmt2(balances.binance_open_positions)),
        Cell::new(format!("{:.2}", balances.binance_open_positions / b_balance).as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Hyperliquid"),
        Cell::new(f.fmt2(hl_balance)),
        Cell::new(f.fmt2(balances.hyperliquid_open_positions)),
        Cell::new(format!("{:.2}", balances.hyperliquid_open_positions / hl_balance).as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Total"),
        Cell::new(f.fmt2(b_balance + hl_balance)),
        Cell::new(f.fmt2(balances.hyperliquid_open_positions + balances.binance_open_positions)),
        Cell::new(
            format!(
                "{:.2}",
                (balances.binance_open_positions + balances.hyperliquid_open_positions)
                    / (b_balance + hl_balance)
            )
            .as_str(),
        ),
    ]));

    Ok(table.to_string())
}
