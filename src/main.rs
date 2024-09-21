mod balances;
mod binance;
mod cli_types;
mod compare_funding_rates;
mod constants;
mod funding_history_table;
mod hyperliquid;
mod orderbook;
mod quote;
mod token_price;
mod util;

use anyhow::{bail, Result};
use balances::{
    build_account_balance_table, build_account_open_positions_table,
    retrieve_account_open_positions,
};
use binance::{
    get_trimmed_quantity, retrieve_binance_order_book, retrieve_binance_past_daily_fh,
    retrieve_step_size,
};
use clap::Parser;
use cli_types::{Cli, Commands};
use compare_funding_rates::build_funding_rate_table;
use funding_history_table::build_past_fr_table;
use hyperliquid::{retrieve_hl_order_book, retrieve_hl_past_daily_fh};
use numfmt::{Formatter, Precision};
use orderbook::retrieve_orderbooks;
use prettytable::{Cell, Row, Table};
use quote::{retrieve_quote_, retrieve_quote_enter};
use token_price::{get_mid_price, retrieve_token_price};
use tokio::try_join;
use util::{calculate_pct_difference, Platform, Side};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    match Cli::parse().command {
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
            let (b_fh, hl_fh) = try_join!(
                retrieve_binance_past_daily_fh(token.clone(), past_days.into()),
                retrieve_hl_past_daily_fh(token.clone(), past_days.into())
            )?;
            let past_daily_rates = build_past_fr_table(b_fh, hl_fh)?;
            println!("{past_daily_rates}");
        }
        Commands::Quote {
            token,
            amount,
            long,
        } => {
            let (b, hl) = retrieve_orderbooks(token).await?;

            let b_spot = get_mid_price(&b)?;
            let hl_spot = get_mid_price(&hl)?;

            let (quote_a, quote_b) = match long {
                Platform::Binance => (
                    retrieve_quote_(b.asks, amount / 2.0, b_spot, Platform::Binance)?,
                    retrieve_quote_(hl.bids, amount / 2.0, hl_spot, Platform::Hyperliquid)?,
                ),
                Platform::Hyperliquid => (
                    retrieve_quote_(hl.asks, amount / 2.0, hl_spot, Platform::Hyperliquid)?,
                    retrieve_quote_(b.bids, amount / 2.0, b_spot, Platform::Binance)?,
                ),
            };

            let slippage_bps = (quote_a.slippage + quote_b.slippage) * 10_000.0;
            let platform_fees_bps =
                ((quote_a.platform_fees + quote_b.platform_fees) / 2.0) * 10_000.0;
            let spread_bps = -(((quote_b.expected_execution_price
                - quote_a.expected_execution_price)
                / quote_a.expected_execution_price)
                * 10_000.0);
            let total_bps = slippage_bps + platform_fees_bps + spread_bps;

            let mut t = Table::new();

            t.add_row(Row::new(vec![Cell::new("Quote (bps)")]));
            t.add_row(Row::new(vec![
                Cell::new("Slippage"),
                Cell::new(&format!("{slippage_bps:.4}")),
            ]));
            t.add_row(Row::new(vec![
                Cell::new("Platform Fees"),
                Cell::new(&format!("{platform_fees_bps:.4}")),
            ]));
            t.add_row(Row::new(vec![
                Cell::new("Spread"),
                Cell::new(&format!("{spread_bps:.4}")),
            ]));
            t.add_row(Row::new(vec![
                Cell::new("Total"),
                Cell::new(&format!("{total_bps:.4}")),
            ]));

            println!("{t}");
        }
        Commands::OrderbookDepth { token } => {
            let (b_orderbook, hl_orderbook) = try_join!(
                retrieve_binance_order_book(token.clone()),
                retrieve_hl_order_book(token.clone()),
            )?;

            let mut f = Formatter::new()
                .precision(Precision::Decimals(0))
                .prefix("$")?
                .separator(',')?;

            let (b_bid, b_ask) = b_orderbook.get_total_depth();
            let (hl_bid, hl_ask) = hl_orderbook.get_total_depth();

            let b_bid = f.fmt2(b_bid).to_string();
            let b_ask = f.fmt2(b_ask).to_string();
            let hl_bid = f.fmt2(hl_bid).to_string();
            let hl_ask = f.fmt2(hl_ask).to_string();

            let text = format!(
                r#"Orderbook Depth {}
Binance: Bids {} — Asks {}
Hyperliquid: Bids {} — Asks {}
"#,
                token, b_bid, b_ask, hl_bid, hl_ask
            );
            println!("{text}");
        }
        Commands::Enter { token, amount } => {
            let ((quote_a, quote_b), step_size) = try_join!(
                // quote_a is short/sell
                retrieve_quote_enter(token.clone(), amount),
                // binance uses a min amount (step size). we are going to use that amount
                // for hyperliquid, too.
                retrieve_step_size(token.clone())
            )?;

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

            // quote costs (bps)
            let quote_slippage = (quote_a.slippage + quote_b.slippage) * 10_000.0;
            let quote_spread = -(((quote_b.expected_execution_price
                - quote_a.expected_execution_price)
                / quote_a.expected_execution_price)
                * 10_000.0);

            // real costs
            let (b_mid_price, hl_mid_price) = if quote_a.platform == Platform::Binance {
                (quote_a.mid_price, quote_b.mid_price)
            } else {
                (quote_b.mid_price, quote_a.mid_price)
            };
            let b_slippage = calculate_pct_difference(b.avg_price, b_mid_price);
            let hl_slippage = calculate_pct_difference(h.avg_price, hl_mid_price);
            let real_slippage = (b_slippage + hl_slippage) * 10_000.0; // bps
            let real_spread = if b.side == Side::Buy {
                -(((h.avg_price - b.avg_price) / b.avg_price) * 10_000.0)
            } else {
                -(((b.avg_price - h.avg_price) / h.avg_price) * 10_000.0)
            };

            println!("order filled one: {:?}", b);
            println!("order filled two: {:?}", h);
            println!("quote slippage: {:.4}", quote_slippage);
            println!("real slippage: {:.4}", real_slippage);
            println!("quote spread: {:.4}", quote_spread);
            println!("real spread: {:.4}", real_spread);
        }
        Commands::Exit { token, amount } => {
            let open_positions_token = retrieve_account_open_positions()
                .await?
                .into_iter()
                .filter(|p| p.coin == token)
                .collect::<Vec<_>>();

            if open_positions_token.len() != 2 {
                bail!("positions do not exist. check again")
            }

            let token_price = retrieve_token_price(token.clone()).await?;
            let intended_size_to_exit = amount / token_price;
            let step_size = retrieve_step_size(token.clone()).await?;
            let trimmed_size = get_trimmed_quantity(intended_size_to_exit, step_size);

            let p1 = &open_positions_token[0];
            let p1_is_buy = p1.direction == "long";

            let size = if trimmed_size > p1.size {
                p1.size
            } else {
                trimmed_size
            };

            match p1.platform {
                Platform::Binance => {
                    let (o1, o2) = try_join!(
                        binance::execute_mkt_order(token.clone(), size, !p1_is_buy),
                        hyperliquid::execute_mkt_order(token.clone(), size, p1_is_buy)
                    )?;
                    println!("{o1:#?} — {o2:#?}");
                }
                Platform::Hyperliquid => {
                    let (o1, o2) = try_join!(
                        hyperliquid::execute_mkt_order(token.clone(), size, !p1_is_buy),
                        binance::execute_mkt_order(token.clone(), size, p1_is_buy)
                    )?;
                    println!("{o1:#?} — {o2:#?}");
                }
            }
        }
    }

    Ok(())
}
