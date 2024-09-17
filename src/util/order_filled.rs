use super::Platform;
use serde::Deserialize;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct OrderFilled {
    pub token: String,
    pub platform: Platform,
    pub size: f64,
    pub avg_price: f64,
    pub side: Side,
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub enum Side {
    Buy,
    Sell,
}
