use super::retrieve_binance_general_info;
use anyhow::Result;

pub async fn retrieve_step_size(token: String) -> Result<f64> {
    let step_size = retrieve_binance_general_info()
        .await?
        .iter()
        .find(|t| t.symbol == format!("{token}USDT"))
        .expect("could not find token")
        .filters
        .step_size;

    Ok(step_size)
}

pub fn get_trimmed_quantity(qty: f64, step_size: f64) -> f64 {
    (qty / step_size).round() * step_size
}
