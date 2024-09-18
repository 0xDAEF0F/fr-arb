use crate::util::calculate_effective_rate;
use anyhow::Result;
use numfmt::Formatter;
use prettytable::{Cell, Row, Table};

pub fn build_past_fr_table(
    binance_daily_rates: Vec<f64>,
    hl_daily_rates: Vec<f64>,
) -> Result<String> {
    let mut f = Formatter::new()
        .precision(numfmt::Precision::Decimals(2))
        .suffix("%")?;

    let mut table = Table::new();

    let mut row = Row::new(vec![Cell::new("Daily rates")]);
    for i in 0..binance_daily_rates.len() {
        row.add_cell(Cell::new(&format!("Day {}", i + 1)));
    }
    table.add_row(row);

    let mut row = Row::new(vec![Cell::new("Binance")]);
    for rate in binance_daily_rates.iter() {
        let annualized_rate = rate * 365.0 * 100.0;
        let fmt_annualized_rate = f.fmt2(annualized_rate);
        row.add_cell(Cell::new(fmt_annualized_rate));
    }
    table.add_row(row);

    let mut row = Row::new(vec![Cell::new("Hyperliquid")]);
    for rate in hl_daily_rates.iter() {
        let annualized_rate = rate * 365.0 * 100.0;
        let fmt_annualized_rate = f.fmt2(annualized_rate);
        row.add_cell(Cell::new(fmt_annualized_rate));
    }
    table.add_row(row);

    let mut row = Row::new(vec![Cell::new("Total")]);
    for (&b, &h) in binance_daily_rates.iter().zip(hl_daily_rates.iter()) {
        let er = calculate_effective_rate(b, h);
        let annualized_rate = er * 365.0 * 100.0;
        let fmt_annualized_rate = f.fmt2(annualized_rate);
        row.add_cell(Cell::new(fmt_annualized_rate));
    }
    table.add_row(row);

    Ok(table.to_string())
}
