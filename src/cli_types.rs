use crate::{constants::MAX_DAYS_QUERY_FUNDING_HISTORY, util::Platform};
use anyhow::{bail, Ok};
use clap::{value_parser, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// retrieves the current balances and open positions
    Balance,
    /// retrieves the current best funding rates
    FundingRates,
    /// retrieves the funding history of a token
    FundingHistory {
        /// name of the token
        #[arg(value_parser = |s: &str| Ok(s.to_uppercase()))]
        token: String,
        /// how many days in the past do you want to inquire? (max 7 days)
        #[arg(short, long, default_value = "7" , value_parser = value_parser!(u8).range(1..=MAX_DAYS_QUERY_FUNDING_HISTORY))]
        past_days: u8,
    },
    /// find out funding rate arbitrage cost to enter/exit into a token
    Quote {
        /// name of the token
        #[arg(value_parser = |s: &str| Ok(s.to_uppercase()))]
        token: String,
        /// amount to quote (USD)
        amount: f64,
        /// name of the platform of where to long (binance or hyperliquid)
        #[arg(long, value_parser = |s: &str| match s.to_lowercase().as_str() {
            "binance" | "b" => Ok(Platform::Binance),
            "hyperliquid" | "hl" | "h" => Ok(Platform::Hyperliquid),
            _ => bail!("Invalid platform. Use 'binance' (or 'b') for Binance, or 'hyperliquid' (or 'hl', 'h') for Hyperliquid")
        })]
        long: Platform,
    },
    /// bid_ask depth of the orderbook for a token in both platforms
    OrderbookDepth {
        /// name of the token
        #[arg(value_parser = |s: &str| Ok(s.to_uppercase()))]
        token: String,
    },
    /// enters from a funding rate position
    Enter {
        /// name of the token
        #[arg(value_parser = |s: &str| Ok(s.to_uppercase()))]
        token: String,
        /// amount to execute (USD)
        amount: f64,
    },
    /// exits from a funding rate position
    Exit {
        /// name of the token
        #[arg(value_parser = |s: &str| Ok(s.to_uppercase()))]
        token: String,
        /// amount to execute (USD)
        amount: f64,
    },
}
