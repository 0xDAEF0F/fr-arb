use anyhow::Result;
use ethers::signers::Signer;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_json::json;

use crate::hyperliquid::get_wallet;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HlAccountRes {
    pub cross_margin_summary: CrossMarginSummary,
    pub asset_positions: Vec<AssetPosition>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrossMarginSummary {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub account_value: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_ntl_pos: f64,
}

#[derive(Deserialize, Debug)]
pub struct AssetPosition {
    pub position: Position,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub coin: String,
    pub szi: String, // negative == Short and positive == Long
    #[allow(dead_code)]
    pub entry_px: String,
    pub unrealized_pnl: String,
    #[serde(
        deserialize_with = "deserialize_number_from_string",
        rename = "positionValue"
    )]
    pub notional: f64,
    pub cum_funding: CumulativeFunding,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CumulativeFunding {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub all_time: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub since_open: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub since_change: f64,
}

pub async fn retrieve_hl_account_info() -> Result<HlAccountRes> {
    let client = Client::new();

    let body = json!({
        "type": "clearinghouseState",
        "user": get_wallet()?.address()
    });

    let res = client
        .post("https://api.hyperliquid.xyz/info".to_string())
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?;

    let hl_account_res: HlAccountRes = res.json().await?;

    Ok(hl_account_res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retrieve_hl_account_info() -> Result<()> {
        let acct_info = retrieve_hl_account_info().await?;

        println!("{acct_info:#?}");

        Ok(())
    }
}
