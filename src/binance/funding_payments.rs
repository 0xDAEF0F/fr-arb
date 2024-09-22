use crate::util::generate_hmac_signature;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingPayment {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub income: f64,
    pub time: u64,
}

pub async fn retrieve_funding_payments(
    token: &str,
    start_time: u64,
) -> Result<Vec<FundingPayment>> {
    let client = Client::new();

    let timestamp = chrono::Utc::now().timestamp_millis();

    let signature = generate_hmac_signature(Some(
        format!(
            "symbol={token}USDT&incomeType=FUNDING_FEE&startTime={start_time}&endTime={timestamp}&timestamp={timestamp}"
        )
        .to_string(),
    ))?;

    let url = format!(
        "https://fapi.binance.com/fapi/v1/income?symbol={}USDT&incomeType=FUNDING_FEE&startTime={}&endTime={}&timestamp={}&signature={}",
        token, start_time, timestamp, timestamp, signature
    );

    let res = client
        .get(url)
        .header("X-MBX-APIKEY", std::env::var("BINANCE_API_KEY")?)
        .send()
        .await?;

    let mut binance_account_res: Vec<FundingPayment> = res.json().await?;

    // descending order
    binance_account_res.sort_by(|a, b| b.time.cmp(&a.time));

    Ok(binance_account_res)
}
