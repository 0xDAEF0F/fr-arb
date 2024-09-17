use anyhow::Result;
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

    Ok(fh)
}

/// `coin` without 'quote'. e.g., BTC
/// `past_days 1..=15` they are validated on the cli parsing
/// returns annualized funding history average
pub async fn retrieve_binance_fh_avg(token: String, past_days: u16) -> Result<f64> {
    let mut fh = retrieve_binance_funding_history(token).await?;
    fh.sort_by(|a, b| b.funding_time.cmp(&a.funding_time));

    // diff between two consecutive funding times `&fh[0] > &fh[1]`
    let funding_interval = (fh[0].funding_time - fh[1].funding_time) / (1000 * 60 * 60);
    let take: u16 = if funding_interval == 8 { 3 } else { 6 };

    let sum: f64 = fh
        .iter()
        .take((past_days * take).into())
        .map(|e| e.funding_rate)
        .sum();

    let mean_fr = (sum * 24.0 * 365.0 * 100.0) / f64::from(past_days * take);

    Ok(mean_fr)
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
        let avg_funding_rate = retrieve_binance_fh_avg(coin, past_days).await?;

        println!("{avg_funding_rate:#?}");

        Ok(())
    }
}
