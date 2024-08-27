use hyperliquid_rust_sdk::InfoClient;
use log::info;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let _info_client = InfoClient::new(None, None).await.unwrap();

    let _fh = funding_history_example(&_info_client).await;
}

async fn funding_history_example(info_client: &InfoClient) {
    let coin = "ETH";

    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    info!(
        "Funding data history for {coin} between timestamps {start_timestamp} and {end_timestamp}: {:#?}",
        info_client.funding_history(coin.to_string(), start_timestamp, Some(end_timestamp)).await.unwrap()
    );
}
