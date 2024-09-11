use super::compare_funding_rate::compare_funding_rates;
use anyhow::Result;
use prettytable::{Cell, Row, Table};

pub async fn build_funding_rate_table() -> Result<String> {
    let fr = compare_funding_rates().await?;
    let top_fr = fr.into_iter().take(8).collect::<Vec<_>>();

    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new("Funding Rates APY")]));
    table.add_row(Row::new(vec![
        Cell::new("Coin"),
        Cell::new("Binance Fr"),
        Cell::new("Hl Fr"),
        Cell::new("Fr Diff"),
    ]));

    for jfr in top_fr {
        // make them yearly and round two decimals
        let b_fr = (jfr.binance_funding_rate * 24.0 * 365.0 * 100.0 * 100.0).round() / 100.0;
        let b_fr = format!("{b_fr}%");
        let hl_fr = (jfr.hyperliquid_funding_rate * 24.0 * 365.0 * 100.0 * 100.0).round() / 100.0;
        let hl_fr = format!("{hl_fr}%");
        let fr_diff = (jfr.funding_rate_difference * 24.0 * 365.0 * 100.0 * 100.0).round() / 100.0;
        let fr_diff = format!("{fr_diff}%");
        table.add_row(Row::new(vec![
            Cell::new(jfr.name.as_str()),
            Cell::new(b_fr.as_str()),
            Cell::new(hl_fr.as_str()),
            Cell::new(fr_diff.as_str()),
        ]));
    }

    Ok(table.to_string())
}
