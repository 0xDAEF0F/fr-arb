use crate::balances::{build_account_balance_table, build_account_open_positions_table};
use crate::binance::retrieve_binance_fh_avg;
use crate::compare_funding_rates::build_funding_rate_table;
use crate::constants::MAX_DAYS_QUERY_FUNDING_HISTORY;
use crate::funding_history::build_avg_fh_table;
use crate::hyperliquid::retrieve_hl_fh_avg;
use crate::util::calculate_effective_rate;
use anyhow::Result;
use rustyline_async::{Readline, ReadlineEvent};
use std::io::Write;
use tokio::try_join;

pub async fn handle_command(
    line: String,
    rl: &mut Readline,
    stdout: &mut impl Write,
) -> Result<()> {
    match line.as_str() {
        // Retrieve Global Balances
        "balance" | "bal" => {
            let (global_balances, open_positions) = try_join!(
                build_account_balance_table(),
                build_account_open_positions_table()
            )?;
            writeln!(stdout, "{}", global_balances)?;
            writeln!(stdout, "{}", open_positions)?;
        }
        // Retrieve Best Current Funding Rates
        "funding rates" | "fr" => {
            let funding_rates_table = build_funding_rate_table().await?;
            writeln!(stdout, "{}", funding_rates_table)?;
        }
        // Query Avg Funding History Up To Last 15 Days Of a Token
        hr if hr.starts_with("funding history") => {
            let coin = hr
                .trim_start_matches("funding history")
                .trim()
                .to_uppercase();
            if coin.is_empty() {
                writeln!(stdout, "Please specify a coin, e.g., 'history rates BTC'")?;
                return Ok(());
            }
            let res = format!(
                r#"How many past days (1-{}) of {} do you want to inquire
about it's funding rate? (average rate)"#,
                MAX_DAYS_QUERY_FUNDING_HISTORY, coin
            );
            writeln!(stdout, "{res}")?;
            if let ReadlineEvent::Line(line) = rl.readline().await? {
                let days: u16 = line.parse()?;

                let (b_avg_fh, hl_avg_fh) = try_join!(
                    retrieve_binance_fh_avg(coin.clone(), days),
                    retrieve_hl_fh_avg(coin.clone(), days)
                )?;

                let effective_rate = calculate_effective_rate(b_avg_fh, hl_avg_fh);

                let table_str = build_avg_fh_table(coin, b_avg_fh, hl_avg_fh, effective_rate)?;

                writeln!(stdout, "{}", table_str)?;
            }
        }
        _ => {}
    }
    Ok(())
}
