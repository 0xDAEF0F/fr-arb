use anyhow::{bail, Result};
use numfmt::Formatter;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingHistory {
    symbol: String,       // token w/ quote at end
    funding_rate: String, // 8 || 4 hours
    funding_time: u64,    // ms timestamp
}

async fn retrieve_binance_funding_history(coin: String) -> Result<Vec<FundingHistory>> {
    let client = Client::new();

    let res = client
        .get(format!(
            "https://fapi.binance.com/fapi/v1/fundingRate?symbol={coin}USDT"
        ))
        .send()
        .await?;

    let fh: Vec<FundingHistory> = res.json().await?;

    Ok(fh)
}

/// `coin` without 'quote'. e.g., BTC
/// `days` > 0 <= 15
pub async fn retrieve_binance_fh_avg(coin: String, past_days: u16) -> Result<String> {
    if past_days > 15 {
        bail!("can only peek up to 15 days")
    } else if past_days == 0 {
        bail!("min 1 days")
    }

    let mut fh = retrieve_binance_funding_history(coin).await?;
    fh.sort_by(|a, b| b.funding_time.cmp(&a.funding_time));

    // diff between two consecutive funding times `&fh[0] > &fh[1]`
    let funding_interval = (&fh[0].funding_time - &fh[1].funding_time) / (1000 * 60 * 60);
    let take: u16 = if funding_interval == 8 { 3 } else { 6 };

    let sum: f64 = fh
        .iter()
        .take((past_days * take).into())
        .map(|e| e.funding_rate.parse::<f64>().unwrap())
        .sum();

    let mean_fr = (sum / f64::from(past_days * take)) * 24.0 * 365.0 * 100.0;

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
