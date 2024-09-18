use crate::{
    binance::{
        account_information::retrieve_binance_account_info,
        funding_rates::retrieve_binance_hourly_funding_rates,
    },
    hyperliquid::{
        account_information::retrieve_hl_account_info,
        funding_rates::retrieve_hl_hourly_funding_rates,
    },
    util::Platform,
};
use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;
use numfmt::Formatter;
use prettytable::{Cell, Row, Table};
use tokio::try_join;

#[derive(Debug)]
pub struct Position {
    pub platform: Platform,
    pub coin: String,      // without quote
    pub direction: String, // short || long
    pub size: f64,         // amount of tokens/cryptocurrency
    pub entry_price: f64,
    pub pnl: f64,
    pub funding_rate: f64, // annualized
}

pub async fn retrieve_account_open_positions() -> Result<Vec<Position>> {
    let info_client = InfoClient::new(None, None).await.unwrap();

    let (binance_acct_info, hyperliquid_acct_info, binance_funding_rates, hl_funding_rates) = try_join!(
        retrieve_binance_account_info(),
        retrieve_hl_account_info(),
        retrieve_binance_hourly_funding_rates(),
        retrieve_hl_hourly_funding_rates(&info_client)
    )?;

    let mut binance_positions: Vec<_> = binance_acct_info
        .positions
        .into_iter()
        .map(|p| {
            let coin = p.symbol.trim_end_matches("USDT").to_string();
            let funding_rate = binance_funding_rates
                .iter()
                .find(|&rate| rate.name == coin)
                .map(|rate| (rate.hourly_funding_rate * 24.0 * 365.0))
                .expect("funding rate not found");
            let direction = p.position_side.to_lowercase();
            let entry_price = (p.notional.abs() - p.unrealized_profit) / p.size;
            Position {
                platform: Platform::Binance,
                coin,
                direction,
                entry_price,
                funding_rate,
                pnl: p.unrealized_profit,
                size: p.size,
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
                platform: Platform::Hyperliquid,
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

    if open_positions.is_empty() {
        return Ok("No open positions.".to_string());
    }

    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new("open positions")]));
    table.add_row(Row::new(vec![
        Cell::new("platform"),
        Cell::new("coin"),
        Cell::new("direction"),
        Cell::new("size (tokens)"),
        Cell::new("entry price"),
        Cell::new("pnl"),
        Cell::new("funding rate (apr)"),
    ]));

    let mut f = Formatter::new()
        .precision(numfmt::Precision::Decimals(2))
        .suffix("%")?;

    for position in open_positions {
        let fmt_annualized_fr = f.fmt2(position.funding_rate * 100.0);
        let fmt_entry_price = format!("${:.5}", position.entry_price);
        let pnl = format!("${:.2}", position.pnl);
        table.add_row(Row::new(vec![
            Cell::new(format!("{:?}", position.platform).as_str()),
            Cell::new(position.coin.as_str()),
            Cell::new(position.direction.as_str()),
            Cell::new(position.size.to_string().as_str()),
            Cell::new(fmt_entry_price.as_str()),
            Cell::new(pnl.as_str()),
            Cell::new(fmt_annualized_fr),
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
