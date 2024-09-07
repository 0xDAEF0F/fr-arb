use crate::util::generate_hmac_signature;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceAccountRes {
    total_wallet_balance: String,
    total_unrealized_profit: String,
    total_margin_balance: String,
    assets: Vec<Asset>,
    positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Asset {
    asset: String,
    wallet_balance: String,
    unrealized_profit: String,
    margin_balance: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    symbol: String,
    position_side: String,
    position_amt: String,
    unrealized_profit: String,
    notional: String,
}

pub async fn retrieve_binance_account_info() -> Result<BinanceAccountRes> {
    let client = Client::new();

    let timestamp = chrono::Utc::now().timestamp_millis();

    let signature = generate_hmac_signature(Some(format!("timestamp={timestamp}").to_string()))?;

    let url = format!(
        "https://fapi.binance.com/fapi/v3/account?timestamp={}&signature={}",
        timestamp, signature
    );

    let res = client
        .get(url)
        .header("X-MBX-APIKEY", std::env::var("BINANCE_API_KEY")?)
        .send()
        .await?;

    let binance_account_res: BinanceAccountRes = res.json().await?;

    Ok(binance_account_res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_binance_account_info() -> Result<()> {
        dotenv::dotenv().ok();

        let acct_info = retrieve_binance_account_info().await?;

        println!("{acct_info:#?}");

        Ok(())
    }
}
