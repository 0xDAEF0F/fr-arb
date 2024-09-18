use super::get_binance_avg_price;
use crate::util::{generate_hmac_signature, OrderFilled, Platform, Side};
use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MktOrderRes {
    pub order_id: u128,
}

pub async fn execute_mkt_order(token: String, size: f64, is_buy: bool) -> Result<OrderFilled> {
    let client = Client::new();
    let timestamp = chrono::Utc::now().timestamp_millis();

    let side = if is_buy { Side::Buy } else { Side::Sell };
    let side_ = format!("{:?}", side).to_uppercase();
    let signature = generate_hmac_signature(Some(format!(
        "symbol={token}USDT&side={side_}&type=MARKET&quantity={size}&timestamp={timestamp}"
    )))?;
    let url = format!("https://fapi.binance.com/fapi/v1/order?symbol={token}USDT&side={side_}&type=MARKET&quantity={size}&timestamp={timestamp}&signature={signature}");

    let res = client
        .post(url)
        .header("X-MBX-APIKEY", std::env::var("BINANCE_API_KEY")?)
        .send()
        .await?;

    if !res.status().is_success() {
        let error = res.text().await?;
        bail!("Binance order failed. {}", error)
    }

    let binance_account_res: MktOrderRes = res.json().await?;

    let avg_price = get_binance_avg_price(token.clone(), binance_account_res.order_id).await?;

    Ok(OrderFilled {
        token,
        platform: Platform::Binance,
        size,
        avg_price,
        side,
    })
}

#[allow(dead_code)]
fn get_trimmed_quantity(qty: f64, step_size: f64) -> f64 {
    (qty / step_size).round() * step_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mkt_order() -> Result<()> {
        dotenv::dotenv().ok();

        let res = execute_mkt_order("TIA".to_string(), 7000.0, true).await?;

        println!("{res:#?}");

        Ok(())
    }
}
