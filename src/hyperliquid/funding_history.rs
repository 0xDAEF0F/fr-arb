use anyhow::Result;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingHistory {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    funding_rate: f64, // hourly
    time: u64, // ms timestamp
}

async fn retrieve_hl_funding_history(token: String) -> Result<Vec<FundingHistory>> {
    let client = Client::new();

    let timestamp = (Utc::now() - Duration::days(15)).timestamp_millis();

    let body = json!({
        "type": "fundingHistory",
        "coin": token,
        "startTime": timestamp,
    });

    let res = client
        .post("https://api.hyperliquid.xyz/info")
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?;

    let fh: Vec<FundingHistory> = res.json().await?;

    Ok(fh)
}

pub async fn retrieve_hl_past_daily_fh(coin: String, past_days: u16) -> Result<Vec<f64>> {
    let mut fh = retrieve_hl_funding_history(coin).await?;
    fh.sort_by(|a, b| b.time.cmp(&a.time));

    let past_daily_fr: Vec<f64> = fh
        .chunks_exact(24)
        .map(|c| c.iter().map(|fh| fh.funding_rate).sum::<f64>())
        .take(past_days.into())
        .collect();

    Ok(past_daily_fr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_hl_funding_history() -> Result<()> {
        let funding_hist = retrieve_hl_funding_history("ETH".to_string()).await?;

        println!("{funding_hist:#?}");

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_hl_fh_avg() -> Result<()> {
        let coin = "BTC".to_string();
        let past_days = 1;
        let btc_ten_day_avg_funding_rate = retrieve_hl_past_daily_fh(coin, past_days).await?;

        println!("{btc_ten_day_avg_funding_rate:#?}");

        Ok(())
    }
}
