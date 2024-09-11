use crate::balances::{build_account_balance_table, build_account_open_positions_table};
use crate::compare_funding_rates::build_funding_rate_table;
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
                r#"How many past days (1-20) of {} do you want to inquire
about it's funding rate? (average rate)"#,
                coin
            );
            writeln!(stdout, "{res}")?;
            if let ReadlineEvent::Line(line) = rl.readline().await? {
                let _days: u16 = line.parse()?;
            }
        }
        _ => {}
    }
    Ok(())
}
