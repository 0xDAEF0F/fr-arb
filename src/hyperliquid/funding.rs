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
    max_leverage: u8,
    only_isolated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingData {
    funding: String,
    open_interest: String,
    prev_day_px: String,
    day_ntl_vlm: String,
    premium: Option<String>,
    oracle_px: String,
    mark_px: String,
    mid_px: Option<String>,
    impact_pxs: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct HyperliquidToken {
    pub name: String,
    pub max_leverage: u8,
    pub sz_decimals: u8,
    pub funding_rate: f64,
    pub open_interest: f64,
}

pub async fn retrieve_hl_tokens(info_client: &InfoClient) -> Vec<HyperliquidToken> {
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

    let mut hyperliquid_tokens = vec![];

    for (token, funding_data) in tokens.into_iter().zip(fr.into_iter()) {
        let funding_rate: f64 = funding_data.funding.parse().unwrap();
        let open_interest: f64 = funding_data.open_interest.parse().unwrap();
        let open_interest: f64 = funding_data.oracle_px.parse::<f64>().unwrap() * open_interest;

        let hyperliquid_token = HyperliquidToken {
            name: token.name,
            max_leverage: token.max_leverage,
            sz_decimals: token.sz_decimals,
            funding_rate,
            open_interest,
        };

        hyperliquid_tokens.push(hyperliquid_token);
    }

    hyperliquid_tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_all_hl_tokens() {
        let info_client = InfoClient::new(None, None).await.unwrap();

        let tokens = retrieve_hl_tokens(&info_client).await;

        println!("{:#?}", tokens);
    }

    #[tokio::test]
    async fn get_top_three_hl_fr() {
        let info_client = InfoClient::new(None, None).await.unwrap();

        let mut tokens = retrieve_hl_tokens(&info_client).await;

        tokens.sort_by(|a, b| {
            b.funding_rate
                .abs()
                .partial_cmp(&a.funding_rate.abs())
                .unwrap()
        });

        let top_three: Vec<_> = tokens.into_iter().take(3).collect();

        println!("{:#?}", top_three);
    }
}
