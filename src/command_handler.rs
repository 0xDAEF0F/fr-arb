use crate::balances::{build_account_balance_table, build_account_open_positions_table};
use crate::binance::{retrieve_binance_fh_avg, retrieve_binance_order_book};
use crate::compare_funding_rates::{build_funding_rate_table, compare_funding_rates};
use crate::constants::MAX_DAYS_QUERY_FUNDING_HISTORY;
use crate::funding_history_table::build_avg_fh_table;
use crate::hyperliquid::{retrieve_hl_fh_avg, retrieve_hl_order_book};
use crate::quote::retrieve_quote;
use crate::util::{calculate_effective_rate, determine_short_based_on_fr, BidAsk, Platform};
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
        // Find Out Funding Rate Arbitrage Cost to Enter Into a Token
        quote if quote.starts_with("quote") => {
            let parts: Vec<&str> = quote.split_whitespace().collect();

            let token = parts[1].to_uppercase();
            let amount: f64 = parts[2].parse()?;

            let jfr = compare_funding_rates()
                .await?
                .into_iter()
                .find(|jfr| jfr.name == token)
                .expect("token must be in joint funding rates");

            let platform = determine_short_based_on_fr(jfr);

            let (short_orderbook, long_orderbook) = match platform {
                Platform::Binance => (
                    retrieve_binance_order_book(token.clone(), BidAsk::Bid).await?,
                    retrieve_hl_order_book(token.clone(), BidAsk::Ask).await?,
                ),
                Platform::Hyperliquid => (
                    retrieve_hl_order_book(token.clone(), BidAsk::Bid).await?,
                    retrieve_binance_order_book(token.clone(), BidAsk::Ask).await?,
                ),
            };

            let quote_a = retrieve_quote(short_orderbook, amount / 2.0)?;
            let quote_b = retrieve_quote(long_orderbook, amount / 2.0)?;

            let slippage_bips = (quote_a.slippage + quote_b.slippage) * 10_000.0;
            let platform_fees_bips = (quote_a.platform_fees + quote_b.platform_fees) * 10_000.0;

            let short_execution_price = quote_a.expected_execution_price;
            let long_execution_price = quote_b.expected_execution_price;

            writeln!(stdout, "slippage: {}", slippage_bips)?;
            writeln!(stdout, "platform fees: {}", platform_fees_bips)?;
            writeln!(
                stdout,
                "total fees (bips): {}",
                slippage_bips + platform_fees_bips
            )?;

            writeln!(
                stdout,
                "short price {:?}: {} — long price {:?}: {}",
                quote_a.platform, short_execution_price, quote_b.platform, long_execution_price
            )?;
        }
        _ => {}
    }
    Ok(())
}
