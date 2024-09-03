mod binance;
mod compare_funding_rates;
mod hyperliquid;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    Ok(())
}
