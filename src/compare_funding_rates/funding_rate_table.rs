use super::compare_funding_rate::compare_funding_rates;
use anyhow::Result;
use prettytable::{Cell, Row, Table};

pub async fn build_funding_rate_table() -> Result<String> {
    let fr = compare_funding_rates().await?;
    let top_fr = fr.into_iter().take(8).collect::<Vec<_>>();

    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new("Open Positions")]));
    table.add_row(Row::new(vec![
        Cell::new("Coin"),
        Cell::new("Binance Fr"),
        Cell::new("Hl Fr"),
        Cell::new("Fr Diff"),
    ]));

    for jfr in top_fr {
        table.add_row(Row::new(vec![
            Cell::new(jfr.name.as_str()),
            Cell::new(jfr.binance_funding_rate.to_string().as_str()),
            Cell::new(jfr.hyperliquid_funding_rate.to_string().as_str()),
            Cell::new(jfr.funding_rate_difference.to_string().as_str()),
        ]));
    }

    Ok(table.to_string())
}
