mod hyperliquid;

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

    // println!("{:#?}", fh);

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
) -> Result<Vec<(String, f64)>> {
    // Token - Funding Rate
    let mut tokens_with_fr: Vec<(String, f64)> = vec![];

    let start_timestamp = (Utc::now() - Duration::minutes(30)).timestamp_millis() as u64;
    println!("{start_timestamp}");

    for token in tokens {
        println!("{token}");
        let fhr = info_client
            .funding_history(token.clone(), start_timestamp, None)
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let fhr = fhr.last().unwrap();
        println!("{:#?}", fhr);

        let funding_rate = fhr.funding_rate.parse::<f64>().unwrap_or(0.0);

        tokens_with_fr.push((fhr.coin.clone(), funding_rate));
    }

    Ok(tokens_with_fr)
}

async fn funding_history_example(info_client: &InfoClient) {
    let coin = "MKR";

    let start_timestamp = 1724803039095;
    let end_timestamp = None;
    info!(
        "{:#?}",
        info_client
            .funding_history(coin.to_string(), start_timestamp, end_timestamp)
            .await
            .unwrap()
    );
}
