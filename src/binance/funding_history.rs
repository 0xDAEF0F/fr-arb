use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingHistory {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    funding_rate: f64,
    funding_time: u64, // ms timestamp
}

async fn retrieve_binance_funding_history(token: String) -> Result<Vec<FundingHistory>> {
    let client = Client::new();

    let res = client
        .get(format!(
            "https://fapi.binance.com/fapi/v1/fundingRate?symbol={token}USDT"
        ))
        .send()
        .await?;

    let fh: Vec<FundingHistory> = res.json().await?;

    if fh.is_empty() {
        bail!("No binance funding history for: {token}");
    }

    Ok(fh)
}

pub async fn retrieve_binance_past_daily_fh(token: String, past_days: u16) -> Result<Vec<f64>> {
    let mut fh = retrieve_binance_funding_history(token).await?;
    fh.sort_by(|a, b| b.funding_time.cmp(&a.funding_time));

    // diff between two consecutive funding times `&fh[0] > &fh[1]`
    let funding_interval = (fh[0].funding_time - fh[1].funding_time) / (1000 * 60 * 60);
    let take: u16 = if funding_interval == 8 { 3 } else { 6 };

    let past_daily_fr: Vec<f64> = fh
        .chunks_exact(take.into())
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
        let funding_hist = retrieve_binance_funding_history("ETH".to_string()).await?;

        println!("{funding_hist:#?}");

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_hl_fh_avg() -> Result<()> {
        let coin = "WIF".to_string();
        let past_days = 3;
        let avg_funding_rate = retrieve_binance_past_daily_fh(coin, past_days).await?;

        println!("{avg_funding_rate:#?}");

        Ok(())
    }
}
