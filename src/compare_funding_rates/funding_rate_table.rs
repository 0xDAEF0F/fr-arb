use super::compare_funding_rate::compare_funding_rates;
use crate::binance::retrieve_token_open_interest;
use anyhow::Result;
use futures::future::try_join_all;
use numfmt::Formatter;
use prettytable::{Cell, Row, Table};
use reqwest::Client;

pub async fn build_funding_rate_table() -> Result<String> {
    let fr = compare_funding_rates().await?;
    let top_fr = fr.into_iter().take(8).collect::<Vec<_>>();

    let http_client = Client::new();

    let all_oi = top_fr
        .iter()
        .map(|jfr| retrieve_token_open_interest(&http_client, format!("{}USDT", jfr.name.clone())))
        .collect::<Vec<_>>();
    let all_oi = try_join_all(all_oi).await?;
    let all_oi_usd: Vec<_> = all_oi
        .iter()
        .enumerate()
        .map(|(idx, oi)| Ok(oi.open_interest.parse::<f64>()? * top_fr[idx].binance_mark_price))
        .collect::<Result<Vec<_>>>()?;

    let mut table = Table::new();
    let mut f = Formatter::new()
        .separator(',')?
        .prefix("$")?
        .precision(numfmt::Precision::Decimals(0));

    table.add_row(Row::new(vec![Cell::new("FR APY")]));
    table.add_row(Row::new(vec![
        Cell::new("Coin"),
        Cell::new("Binance Fr"),
        Cell::new("Binance OI"),
        Cell::new("Hl Fr"),
        Cell::new("Hl OI"),
        Cell::new("Fr Diff"),
    ]));

    for (i, jfr) in top_fr.into_iter().enumerate() {
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
            Cell::new(f.fmt2(all_oi_usd[i])),
            Cell::new(hl_fr.as_str()),
            Cell::new(f.fmt2(jfr.hyperliquid_open_interest.parse::<f64>()?)),
            Cell::new(fr_diff.as_str()),
        ]));
    }

    Ok(table.to_string())
}
