use crate::util::LimitOrder;
use anyhow::bail;
use anyhow::Result;

#[derive(Debug)]
pub struct Quote {
    pub execution_price: f64,
    pub slippage_in_bips: f64,
    pub slippage_in_usd: f64,
}

pub fn retrieve_quote_for_purchase(orderbook: Vec<LimitOrder>, amount: f64) -> Result<Quote> {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    if orderbook.is_empty() {
        bail!("empty orderbook")
    }

    let first_price = orderbook[0].price;
    for bid_ask in orderbook {
        let price = bid_ask.price;
        let size = bid_ask.size;
        let total = price * size;

        // check if total is greater that remaining amount
        if total > remaining_amount {
            // partially take from that order
            let quantity_needed = remaining_amount / price;

            total_cost += quantity_needed * price;
            total_quantity += quantity_needed;
            remaining_amount -= quantity_needed * price;

            break;
        }

        total_cost += total;
        total_quantity += size;
        remaining_amount -= total;
    }

    // order book does not have enough orders for the amount
    if remaining_amount > 0.0 {
        bail!("orderbook can't cover the amount to buy/sell")
    }

    let execution_price = total_cost / total_quantity;

    let slippage_between_spot_price_and_execution =
        calculate_pct_difference(execution_price, first_price);
    let dollar_amt_slippage = amount * (slippage_between_spot_price_and_execution / 10_000.0);

    let quote_price = Quote {
        execution_price,
        slippage_in_bips: slippage_between_spot_price_and_execution,
        slippage_in_usd: (dollar_amt_slippage * 100.0).round() / 100.0,
    };

    Ok(quote_price)
}

// returns BPS
fn calculate_pct_difference(execution_price: f64, expected_price: f64) -> f64 {
    let res = ((execution_price - expected_price).abs() / expected_price) * 10_000.0;
    (res * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::BidAsk;
    use crate::{binance::retrieve_binance_order_book, hyperliquid::retrieve_hl_order_book};

    #[tokio::test]
    async fn test_quote_binance() {
        let orderbook = retrieve_binance_order_book("ETHUSDT".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let quote = retrieve_quote_for_purchase(orderbook, 9_000_000.0).unwrap();

        println!("quote: {quote:#?}");
    }

    #[tokio::test]
    async fn test_quote_hl() {
        let orderbook = retrieve_hl_order_book("BTC".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let quote = retrieve_quote_for_purchase(orderbook, 1_000_000.0).unwrap();

        println!("quote: {quote:#?}");
    }

    #[tokio::test]
    async fn test_depth_hl_orderbook() {
        let result = retrieve_hl_order_book("BTC".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let total_value_of_order_book: f64 = result
            .iter()
            .fold(0.0, |acc, curr| acc + (curr.price * curr.size));

        println!("Total value of orderbook: {}", total_value_of_order_book);
    }

    #[tokio::test]
    async fn test_pct_diff() {
        let diff = calculate_pct_difference(2600.0, 2550.0); // buying
        let diff2 = calculate_pct_difference(2550.0, 2600.0); // selling

        println!("buy example: {diff}");
        println!("sell example: {diff2}");
    }
}
