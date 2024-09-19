use super::Platform;

#[derive(Clone, Debug)]
pub struct Orderbook {
    pub platform: Platform,
    pub bids: Vec<LimitOrder>,
    pub asks: Vec<LimitOrder>,
}

#[derive(Clone, Debug)]
pub struct LimitOrder {
    pub price: f64,
    pub size: f64,
}
