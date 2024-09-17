use crate::util::generate_hmac_signature;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
struct BinanceOrder {
    order_id: u128,
    symbol: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    avg_price: f64,
    orig_qty: String,
}

pub async fn get_binance_avg_price(token: String, order_id: u128) -> Result<f64> {
    let o = get_binance_order(token, order_id).await?;
    Ok(o.avg_price)
}

async fn get_binance_order(token: String, order_id: u128) -> Result<BinanceOrder> {
    let client = Client::new();
    let timestamp = chrono::Utc::now().timestamp_millis();

    let signature = generate_hmac_signature(Some(format!(
        "symbol={token}USDT&orderId={order_id}&timestamp={timestamp}"
    )))?;
    let url = format!(
        "https://fapi.binance.com/fapi/v1/order?symbol={token}USDT&orderId={order_id}&timestamp={timestamp}&signature={signature}"
    );

    let res = client
        .get(url)
        .header("X-MBX-APIKEY", std::env::var("BINANCE_API_KEY")?)
        .send()
        .await?;

    let bo: BinanceOrder = res.json().await?;

    Ok(bo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_order() -> Result<()> {
        dotenv::dotenv().ok();

        let res = get_binance_order("TIA".to_string(), 7371815734).await?;

        println!("{res:#?}");

        Ok(())
    }
}
