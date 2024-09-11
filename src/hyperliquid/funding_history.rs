use anyhow::{bail, Result};
use chrono::{Duration, Utc};
use numfmt::Formatter;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingHistory {
    coin: String,         // token all caps
    funding_rate: String, // hourly
    time: u64,            // ms timestamp
}

async fn retrieve_hl_funding_history(coin: String) -> Result<Vec<FundingHistory>> {
    let client = Client::new();

    let timestamp = (Utc::now() - Duration::days(20)).timestamp_millis();

    let body = json!({
        "type": "fundingHistory",
        "coin": coin,
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

/// `coin` without 'quote'. e.g., BTC
/// `days` > 0 <= 15
pub async fn retrieve_hl_fh_avg(coin: String, past_days: u16) -> Result<String> {
    if past_days > 15 {
        bail!("can only peek up to 15 days")
    } else if past_days == 0 {
        bail!("min 1 days")
    }

    let mut fh = retrieve_hl_funding_history(coin).await?;
    fh.sort_by(|a, b| b.time.cmp(&a.time));

    let sum: f64 = fh
        .iter()
        .take((past_days * 24).into())
        .map(|e| e.funding_rate.parse::<f64>().unwrap())
        .sum();

    let mean_fr = (sum / f64::from(past_days * 24)) * 24.0 * 365.0 * 100.0;

    let mut f = Formatter::new()
        .precision(numfmt::Precision::Decimals(2))
        .suffix("%")?;

    let mean_fr = f.fmt2(mean_fr).to_string();

    Ok(mean_fr)
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
        let coin = "AAVE".to_string();
        let past_days = 1;
        let btc_ten_day_avg_funding_rate = retrieve_hl_fh_avg(coin, past_days).await?;

        println!("{btc_ten_day_avg_funding_rate:#?}");

        Ok(())
    }
}
