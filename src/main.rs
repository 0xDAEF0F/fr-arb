mod balances;
mod binance;
mod cli_types;
mod compare_funding_rates;
mod constants;
mod funding_history_table;
mod hyperliquid;
mod quote;
mod util;

use anyhow::Result;
use balances::{build_account_balance_table, build_account_open_positions_table};
use binance::{retrieve_binance_fh_avg, retrieve_binance_general_info};
use clap::Parser;
use cli_types::{Cli, Commands};
use compare_funding_rates::build_funding_rate_table;
use funding_history_table::build_avg_fh_table;
use hyperliquid::retrieve_hl_fh_avg;
use quote::retrieve_quote;
use tokio::try_join;
use util::{calculate_effective_rate, Platform};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Balance => {
            let (global_balances, open_positions) = try_join!(
                build_account_balance_table(),
                build_account_open_positions_table()
            )?;
            println!("{global_balances}");
            println!("{open_positions}");
        }
        Commands::FundingRates => {
            let funding_rates_table = build_funding_rate_table().await?;
            println!("{funding_rates_table}");
        }
        Commands::FundingHistory { token, past_days } => {
            let (b_avg_fh, hl_avg_fh) = try_join!(
                retrieve_binance_fh_avg(token.clone(), past_days.into()),
                retrieve_hl_fh_avg(token.clone(), past_days.into())
            )?;
            let effective_rate = calculate_effective_rate(b_avg_fh, hl_avg_fh);
            let table_str = build_avg_fh_table(token, b_avg_fh, hl_avg_fh, effective_rate)?;
            println!("{table_str}");
        }
        Commands::Quote { token, amount } => {
            let (quote_a, quote_b) = retrieve_quote(token, amount).await?;

            let slippage_bips = (quote_a.slippage + quote_b.slippage) * 10_000.0;
            let platform_fees_bips = (quote_a.platform_fees + quote_b.platform_fees) * 10_000.0;

            let short_execution_price = quote_a.expected_execution_price;
            let long_execution_price = quote_b.expected_execution_price;

            println!("slippage: {}", slippage_bips);
            println!("platform fees: {}", platform_fees_bips);
            println!("total fees (bips): {}", slippage_bips + platform_fees_bips);

            println!(
                "short price {:?}: {} — long price {:?}: {}",
                quote_a.platform, short_execution_price, quote_b.platform, long_execution_price
            );
        }
        Commands::ManagePosition {
            token,
            amount,
            position_action: _,
        } => {
            // quote_a is short/sell
            let (quote_a, _quote_b) = retrieve_quote(token.clone(), amount).await?;

            // binance uses a min amount (step size). we are going to use that amount
            // for hyperliquid, too.
            let step_size = retrieve_binance_general_info()
                .await?
                .iter()
                .find(|t| t.symbol == format!("{token}USDT"))
                .expect("could not find token")
                .filters
                .step_size;

            // the size we are about to long/short
            let trimmed_qty = (quote_a.size / step_size).round() * step_size;

            let (b, h) = match quote_a.platform {
                Platform::Binance => try_join!(
                    binance::execute_mkt_order(token.clone(), trimmed_qty, false),
                    hyperliquid::execute_mkt_order(token.clone(), trimmed_qty, true)
                )?,
                Platform::Hyperliquid => try_join!(
                    binance::execute_mkt_order(token.clone(), trimmed_qty, true),
                    hyperliquid::execute_mkt_order(token.clone(), trimmed_qty, false)
                )?,
            };

            println!("order filled one: {:?}", b);
            println!("order filled two: {:?}", h);
        }
    }

    Ok(())
}
