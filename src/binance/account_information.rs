use crate::util::generate_hmac_signature;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceAccountRes {
    pub total_margin_balance: String,
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub position_side: String, // SHORT || LONG
    pub position_amt: String,
    pub unrealized_profit: String,
    pub notional: String,
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

    let mut binance_account_res: BinanceAccountRes = res.json().await?;

    for p in binance_account_res.positions.iter_mut() {
        if p.position_amt.parse::<f64>()?.is_sign_negative() {
            p.position_side = "SHORT".to_string();
        } else {
            p.position_side = "LONG".to_string();
        }
    }

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
