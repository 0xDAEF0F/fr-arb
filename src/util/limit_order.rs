use super::Platform;

#[derive(Clone, Debug)]
pub struct LimitOrder {
    pub price: f64,
    pub size: f64,
    pub platform: Platform,
}
