mod hyperliquid;

use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let _info_client = InfoClient::new(None, None).await.unwrap();

    Ok(())
}
