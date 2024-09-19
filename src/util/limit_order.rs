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

impl Orderbook {
    pub fn get_total_depth(&self) -> (f64, f64) {
        let bids_total = self
            .bids
            .iter()
            .fold(0.0, |acc, lo| acc + lo.price * lo.size);
        let asks_total = self
            .asks
            .iter()
            .fold(0.0, |acc, lo| acc + lo.price * lo.size);
        (bids_total, asks_total)
    }
}
