use crate::{
    binance::general_info::retrieve_binance_general_info,
    quote::retrieve_quote,
    util::{generate_hmac_signature, BidAsk},
};
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

use super::retrieve_binance_order_book;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NewOrderRes {
    avg_price: f64,
    side: String, // BUY || SELL
    #[serde(rename = "symbol")]
    token: String,
}

async fn execute_mkt_order(token: String, amt: f64, is_buy: bool) -> Result<()> {
    let client = Client::new();
    let timestamp = chrono::Utc::now().timestamp_millis();

    let ba = if is_buy { BidAsk::Ask } else { BidAsk::Bid };
    let orderbook = retrieve_binance_order_book(format!("{token}USDT"), ba).await?;
    let quote = retrieve_quote(orderbook, amt)?;

    let step_size = retrieve_binance_general_info(&client)
        .await?
        .iter()
        .find(|t| t.symbol == format!("{token}USDT"))
        .expect("could not find token")
        .filters
        .step_size;

    let quantity = get_trimmed_quantity(quote.size, step_size);
    println!("qty: {quantity}");
    let side = if is_buy { "BUY" } else { "SELL" };
    let signature = generate_hmac_signature(Some(
        format!(
            "symbol={token}USDT&side={side}&type=MARKET&quantity={quantity}&timestamp={timestamp}"
        )
        .to_string(),
    ))?;
    let url = format!("https://fapi.binance.com/fapi/v1/order?symbol={token}USDT&side={side}&type=MARKET&quantity={quantity}&timestamp={timestamp}&signature={signature}");

    let res = client
        .post(url)
        .header("X-MBX-APIKEY", std::env::var("BINANCE_API_KEY")?)
        .send()
        .await?;

    let text = res.text().await?;

    println!("{}", text);

    // let binance_account_res: NewOrderRes = res.json().await?;

    // Ok(binance_account_res)
    Ok(())
}

fn get_trimmed_quantity(qty: f64, step_size: f64) -> f64 {
    (qty / step_size).round() * step_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mkt_order() -> Result<()> {
        dotenv::dotenv().ok();

        let res = execute_mkt_order("TIA".to_string(), 11.0, true).await?;

        // println!("{res:#?}");

        Ok(())
    }
}
