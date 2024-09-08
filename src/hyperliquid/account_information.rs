use crate::constants::HYPERLIQUID_PUBLIC_KEY;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HlAccountRes {
    cross_margin_summary: CrossMarginSummary,
    asset_positions: Vec<AssetPosition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CrossMarginSummary {
    account_value: String,
    total_ntl_pos: String, // notional position
    total_margin_used: String,
}

#[derive(Debug, Deserialize)]
struct AssetPosition {
    position: Position,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    coin: String,
    szi: String, // negative == Short and positive == Long
    entry_px: String,
    position_value: String,
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
