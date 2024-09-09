use crate::constants::HYPERLIQUID_PUBLIC_KEY;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HlAccountRes {
    pub cross_margin_summary: CrossMarginSummary,
    pub asset_positions: Vec<AssetPosition>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrossMarginSummary {
    pub account_value: String,
    total_ntl_pos: String, // notional position
    total_margin_used: String,
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
    pub entry_px: String,
    pub position_value: String,
    pub unrealized_pnl: String,
}

pub async fn retrieve_hl_account_info() -> Result<HlAccountRes> {
    let client = Client::new();

    let body = json!({
        "type": "clearinghouseState",
        "user": HYPERLIQUID_PUBLIC_KEY
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
