use crate::util::generate_hmac_signature;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceAccountRes {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_margin_balance: f64,
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub position_side: String, // SHORT || LONG
    #[serde(
        deserialize_with = "deserialize_number_from_string",
        rename = "positionAmt"
    )]
    pub size: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub unrealized_profit: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub notional: f64,
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
        if p.size.is_sign_negative() {
            p.position_side = "SHORT".to_string();
            p.size = p.size.abs();
            p.notional = p.notional.abs();
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
    async fn test_binance_account_info() -> Result<()> {
        dotenv::dotenv().ok();

        let client = Client::new();
        let timestamp = chrono::Utc::now().timestamp_millis();
        let signature =
            generate_hmac_signature(Some(format!("timestamp={timestamp}").to_string()))?;
        let url = format!(
            "https://fapi.binance.com/fapi/v3/account?timestamp={}&signature={}",
            timestamp, signature
        );
        let res = client
            .get(url)
            .header("X-MBX-APIKEY", std::env::var("BINANCE_API_KEY")?)
            .send()
            .await?;

        let res = res.text().await?;

        println!("{}", res);

        Ok(())
    }
}
