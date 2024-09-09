use anyhow::Result;
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
    name: String,
    max_leverage: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingData {
    funding: String,
    open_interest: String,
}

#[derive(Debug)]
pub struct HyperliquidToken {
    pub name: String,
    pub max_leverage: u8,
    pub hourly_funding_rate: f64,
}

pub async fn retrieve_hl_hourly_funding_rates(
    info_client: &InfoClient,
) -> Result<Vec<HyperliquidToken>> {
    let data = json!({
        "type": "metaAndAssetCtxs"
    });
    let request = info_client
        .http_client
        .post("/info", data.to_string())
        .await?;

    let ds: Response = serde_json::from_str(&request)?;

    let tokens = ds.0.universe;
    let fr = ds.1;

    let mut hyperliquid_tokens = vec![];

    for (token, funding_data) in tokens.into_iter().zip(fr.into_iter()) {
        let funding_rate: f64 = funding_data.funding.parse()?;

        let hyperliquid_token = HyperliquidToken {
            name: token.name,
            max_leverage: token.max_leverage,
            hourly_funding_rate: funding_rate,
        };

        hyperliquid_tokens.push(hyperliquid_token);
    }

    Ok(hyperliquid_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_all_hl_tokens() -> Result<()> {
        let info_client = InfoClient::new(None, None).await.unwrap();

        let tokens = retrieve_hl_hourly_funding_rates(&info_client).await?;

        println!("{:#?}", tokens);

        Ok(())
    }

    #[tokio::test]
    async fn get_specific_hl_fr() -> Result<()> {
        let info_client = InfoClient::new(None, None).await.unwrap();

        let tokens = retrieve_hl_hourly_funding_rates(&info_client).await?;

        let token: Vec<_> = tokens
            .into_iter()
            .filter(|t| t.name == "BTC".to_string())
            .collect();

        let token = &token[0];

        println!("{token:#?}");

        Ok(())
    }
}
