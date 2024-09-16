use crate::constants::MAX_DAYS_QUERY_FUNDING_HISTORY;
use clap::{value_parser, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// retrieves the balances and open positions
    Balance,
    /// retrieves the current funding rates
    FundingRates,
    /// retrieves the funding history of a token
    FundingHistory {
        /// name of the token
        #[arg(short, long, value_parser = |s: &str| anyhow::Ok(s.to_uppercase()))]
        token: String,
        /// how many days in the past do you
        /// want to inquire? (max 15 days)
        #[arg(short, long, value_parser = value_parser!(u8).range(1..=MAX_DAYS_QUERY_FUNDING_HISTORY))]
        past_days: u8,
    },
    /// find out funding rate arbitrage cost to enter into a token
    Quote {
        /// name of the token
        #[arg(short, long, value_parser = |s: &str| anyhow::Ok(s.to_uppercase()))]
        token: String,
        /// amount to quote (USD)
        #[arg(short, long)]
        amount: f64,
    },
    /// enters or sells from a funding rate position
    ManagePosition {
        /// name of the token
        #[arg(short, long, value_parser = |s: &str| anyhow::Ok(s.to_uppercase()))]
        token: String,
        /// amount to execute (USD)
        #[arg(short, long)]
        amount: f64,
        #[arg(short, long, value_enum)]
        position_action: PositionAction,
    },
}

#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum PositionAction {
    Enter,
    Exit,
}
