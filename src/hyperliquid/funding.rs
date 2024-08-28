use hyperliquid_rust_sdk::InfoClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct Response(Universe, Vec<FundingData>);

#[derive(Debug, Serialize, Deserialize)]
struct Universe {
    universe: Vec<Token>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Token {
    sz_decimals: u8,
    name: String,
    max_leverage: u32,
    only_isolated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingData {
    funding: Option<String>,
    open_interest: String,
    prev_day_px: String,
    day_ntl_vlm: String,
    premium: Option<String>,
    oracle_px: String,
    mark_px: String,
    mid_px: Option<String>,
    impact_pxs: Option<Vec<String>>,
}

pub async fn idk(info_client: &InfoClient) {
    let data = json!({
        "type": "metaAndAssetCtxs"
    });
    let request = info_client
        .http_client
        .post("/info", data.to_string())
        .await
        .unwrap();

    let ds: Response = serde_json::from_str(&request).unwrap();

    let tokens = ds.0.universe;
    let fr = ds.1;

    for (token, funding_data) in tokens.into_iter().zip(fr.into_iter()) {
        println!("token: {:#?} --- funding_data: {:#?}", token, funding_data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_idk() {
        let info_client = InfoClient::new(None, None).await.unwrap();
        idk(&info_client).await;
    }
}
