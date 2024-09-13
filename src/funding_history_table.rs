use anyhow::Result;
use numfmt::Formatter;
use prettytable::{Cell, Row, Table};

pub fn build_avg_fh_table(
    coin: String,
    b_avg_fh: f64,
    hl_avg_fh: f64,
    effective_rate: f64,
) -> Result<String> {
    let mut f = Formatter::new()
        .precision(numfmt::Precision::Decimals(2))
        .suffix("%")?;

    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new(coin.to_lowercase().as_str()),
        Cell::new("FR (APR)"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Binance"),
        Cell::new(f.fmt2(b_avg_fh)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Hyperliquid"),
        Cell::new(f.fmt2(hl_avg_fh)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Effective Rate"),
        Cell::new(f.fmt2(effective_rate)),
    ]));

    Ok(table.to_string())
}
