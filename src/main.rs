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
use quote::{get_expected_execution_price, retrieve_quote_, retrieve_quote_enter};
use token_price::{get_mid_price, retrieve_token_price};
use tokio::try_join;
use util::{calculate_pct_difference, format_token, Platform, Side};

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
            let (b_token, hl_token) = format_token(&token);
            let (b_fh, hl_fh) = try_join!(
                retrieve_binance_past_daily_fh(b_token, past_days.into()),
                retrieve_hl_past_daily_fh(hl_token, past_days.into())
            )?;
            let past_daily_rates = build_past_fr_table(b_fh, hl_fh)?;
            println!("{past_daily_rates}");
        }
        Commands::Quote {
            token,
            amount,
            long,
        } => {
            let (b, hl) = retrieve_orderbooks(&token).await?;

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

            let (b_slippage, hl_slippage) = if quote_a.platform == Platform::Binance {
                (quote_a.slippage, quote_b.slippage)
            } else {
                (quote_b.slippage, quote_a.slippage)
            };

            let platform_fees_bps =
                ((quote_a.platform_fees + quote_b.platform_fees) / 2.0) * 10_000.0;
            let spread_bps = -(((quote_b.expected_execution_price
                - quote_a.expected_execution_price)
                / quote_a.expected_execution_price)
                * 10_000.0);
            let total_fees_bps = ((b_slippage + hl_slippage) * 10_000.0) + platform_fees_bps;

            let mut t = Table::new();

            t.add_row(Row::new(vec![Cell::new("Quote (bps)")]));
            t.add_row(Row::new(vec![
                Cell::new("Slippage Binance"),
                Cell::new(&format!("{:.4}", b_slippage * 10_000.0)),
            ]));
            t.add_row(Row::new(vec![
                Cell::new("Slippage Hyperliquid"),
                Cell::new(&format!("{:.4}", hl_slippage * 10_000.0)),
            ]));
            t.add_row(Row::new(vec![
                Cell::new("Platform Fees"),
                Cell::new(&format!("{platform_fees_bps:.4}")),
            ]));
            t.add_row(Row::new(vec![
                Cell::new("Total Fees"),
                Cell::new(&format!("{:.4}", total_fees_bps)),
            ]));

            println!("Spread: {:.4}", spread_bps);
            println!("{t}");
        }
        Commands::OrderbookDepth { token } => {
            let (b_orderbook, hl_orderbook) = try_join!(
                retrieve_binance_order_book(&token),
                retrieve_hl_order_book(&token),
            )?;

            let (b_bid, b_ask) = b_orderbook.get_total_depth();
            let (hl_bid, hl_ask) = hl_orderbook.get_total_depth();

            let mut f = Formatter::new()
                .precision(Precision::Decimals(0))
                .prefix("$")?
                .separator(',')?;

            let text = format!(
                r#"Orderbook Depth {}
Binance: Bids {} — Asks {}
Hyperliquid: Bids {} — Asks {}
"#,
                token,
                f.fmt2(b_bid).to_string(),
                f.fmt2(b_ask).to_string(),
                f.fmt2(hl_bid).to_string(),
                f.fmt2(hl_ask).to_string()
            );
            println!("{text}");
        }
        Commands::Execute {
            token,
            size,
            long,
            max_slippage,
        } => {
            let (b, hl) = retrieve_orderbooks(&token).await?;

            let b_mp = get_mid_price(&b)?;
            let hl_mp = get_mid_price(&hl)?;

            let ((buy_expected_px, buy_mp), (sell_expected_px, sell_mp)) = match long {
                Platform::Binance => (
                    (get_expected_execution_price(b.asks, size / 2.0)?, b_mp),
                    (get_expected_execution_price(hl.bids, size / 2.0)?, hl_mp),
                ),
                Platform::Hyperliquid => (
                    (get_expected_execution_price(hl.asks, size / 2.0)?, hl_mp),
                    (get_expected_execution_price(b.bids, size / 2.0)?, b_mp),
                ),
            };

            let buy_slippage = calculate_pct_difference(buy_expected_px, buy_mp);
            let sell_slippage = calculate_pct_difference(sell_expected_px, sell_mp);

            let total_slippage_bps = (buy_slippage + sell_slippage) * 10_000.0;

            if total_slippage_bps > max_slippage {
                bail!(
                    "Total slippage of {:.4} exceeds maximum slippage of {:.4}.",
                    total_slippage_bps,
                    max_slippage
                )
            }

            let (b_token, hl_token) = format_token(&token);

            let (b, h) = match long {
                Platform::Binance => try_join!(
                    binance::execute_mkt_order(b_token, size / 2.0, true),
                    hyperliquid::execute_mkt_order(hl_token, size / 2.0, false)
                )?,
                Platform::Hyperliquid => try_join!(
                    binance::execute_mkt_order(b_token, size / 2.0, false),
                    hyperliquid::execute_mkt_order(hl_token, size / 2.0, true)
                )?,
            };

            // quote costs (bps)
            let quote_slippage = total_slippage_bps;
            let quote_spread =
                -(((sell_expected_px - buy_expected_px) / buy_expected_px) * 10_000.0);

            // real costs
            let b_slippage = calculate_pct_difference(b.avg_price, b_mp);
            let hl_slippage = calculate_pct_difference(h.avg_price, hl_mp);
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
    }

    Ok(())
}
