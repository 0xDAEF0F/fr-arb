use crate::balances::{build_account_balance_table, build_account_open_positions_table};
use crate::compare_funding_rates::build_funding_rate_table;
use anyhow::Result;
use std::io::Write;
use tokio::try_join;

pub async fn handle_command(line: String, stdout: &mut impl Write) -> Result<()> {
    match line.as_str() {
        "balance" | "bal" => {
            let (global_balances, open_positions) = try_join!(
                build_account_balance_table(),
                build_account_open_positions_table()
            )?;
            writeln!(stdout, "{}", global_balances)?;
            writeln!(stdout, "{}", open_positions)?;
        }
        "funding rates" | "fr" => {
            let funding_rates_table = build_funding_rate_table().await?;
            writeln!(stdout, "{}", funding_rates_table)?;
        }
        hr if hr.starts_with("history rates") => {
            let coin = hr.trim_start_matches("history rates").trim().to_uppercase();
            if coin.is_empty() {
                writeln!(stdout, "Please specify a coin, e.g., 'history rates BTC'")?;
                return Ok(());
            }
            // give out some averages of the funding rates history
            writeln!(stdout, "{}", coin)?;
        }
        _ => {}
    }
    Ok(())
}
