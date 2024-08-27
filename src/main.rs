use anyhow::Result;
use chrono::{Duration, Utc};
use hyperliquid_rust_sdk::InfoClient;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let info_client = InfoClient::new(None, None).await.unwrap();

    // let fh = funding_history_example(&info_client).await;

    let tokens = get_tokens();

    let token_plus_fr = retrieve_funding_rates_from_tokens(tokens.unwrap(), &info_client).await?;

    println!("{:#?}", token_plus_fr);

    // println!("{:#?}", tokens);

    Ok(())
}

fn get_tokens() -> Result<Vec<String>> {
    let contents = std::fs::read_to_string("data/tokens.txt")?;

    let tokens: Vec<String> = contents.split(',').map(String::from).collect();

    Ok(tokens)
}

async fn retrieve_funding_rates_from_tokens(
    tokens: Vec<String>,
    info_client: &InfoClient,
) -> Result<Vec<(String, String)>> {
    // Token - Funding Rate
    let mut tokens_with_fr: Vec<(String, String)> = vec![];

    let start_timestamp = (Utc::now() - Duration::hours(1)).timestamp_millis() as u64;
    println!("{start_timestamp}");

    for token in tokens {
        let fhr = info_client
            .funding_history(token.clone(), start_timestamp, None)
            .await?;

        println!("{token}");
        println!("{:#?}", fhr);

        let fhr = fhr.first().unwrap();

        tokens_with_fr.push((fhr.coin.clone(), fhr.funding_rate.clone()));
    }

    Ok(tokens_with_fr)
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
