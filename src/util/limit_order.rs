use super::Platform;

#[derive(Clone, Debug)]
pub struct Orderbook {
    pub platform: Platform,
    pub limit_orders: Vec<LimitOrder>,
}

#[derive(Clone, Debug)]
pub struct LimitOrder {
    pub price: f64,
    pub size: f64,
}
