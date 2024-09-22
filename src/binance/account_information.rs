use crate::{
    binance::retrieve_funding_payments, hyperliquid::get_wallet, util::generate_hmac_signature,
};
use anyhow::Result;
use ethers::signers::Signer;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, UserFillsResponse};
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
    #[serde(default)]
    pub funding: f64,
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
    let user_fills_hl = retrieve_user_fills_hl().await?;

    for p in binance_account_res.positions.iter_mut() {
        if p.size.is_sign_negative() {
            p.position_side = "SHORT".to_string();
            p.size = p.size.abs();
            p.notional = p.notional.abs();
        } else {
            p.position_side = "LONG".to_string();
        }

        let symbol = p.symbol.trim_end_matches("USDT").trim_end_matches("USDC");
        let timestamp_of_interest = user_fills_hl
            .iter()
            .find(|uf| &uf.coin == symbol && is_directions_match(&p.position_side, &uf.dir))
            .expect("token should be there")
            .time;
        let funding_fees = retrieve_funding_payments(symbol, timestamp_of_interest).await?;
        let funding_fees = funding_fees.iter().map(|fee| fee.income).sum::<f64>();

        p.funding = funding_fees;
    }

    Ok(binance_account_res)
}

fn is_directions_match(b_dir: &str, hl_dir: &str) -> bool {
    match b_dir {
        "LONG" => hl_dir == "Open Short",
        "SHORT" => hl_dir == "Open Long",
        _ => panic!("unreachable"),
    }
}

async fn retrieve_user_fills_hl() -> Result<Vec<UserFillsResponse>> {
    let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;

    let user_fills = info_client.user_fills(get_wallet()?.address()).await?;
    let user_fills = user_fills
        .into_iter()
        .filter(|uf| &uf.start_position == "0.0")
        .collect();

    Ok(user_fills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_binance_account_info() -> Result<()> {
        dotenv::dotenv().ok();

        // let b = retrieve_binance_account_info().await?;

        // println!("{:#?}", b);

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
