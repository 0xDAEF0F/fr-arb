use crate::constants::BINANCE_FEE;
use crate::constants::HYPERLIQUID_FEE;
use crate::util::LimitOrder;
use crate::util::Platform;
use anyhow::bail;
use anyhow::Result;

#[derive(Debug)]
pub struct Quote {
    pub platform: Platform,
    pub execution_price: f64,
    pub platform_fees: f64, // decimal pct
    pub slippage: f64,      // decimal pct
}

pub fn retrieve_quote(orderbook: Vec<LimitOrder>, amount: f64) -> Result<Quote> {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    if orderbook.is_empty() {
        bail!("empty orderbook")
    }

    let first_price = orderbook[0].price;
    let platform = orderbook[0].platform;
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

    let quote = Quote {
        execution_price,
        platform,
        slippage: calculate_pct_difference(execution_price, first_price),
        platform_fees: match platform {
            Platform::Binance => BINANCE_FEE,
            Platform::Hyperliquid => HYPERLIQUID_FEE,
        },
    };

    Ok(quote)
}

// returns in decimal percentage
fn calculate_pct_difference(execution_price: f64, first_price: f64) -> f64 {
    (execution_price - first_price).abs() / first_price
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{BidAsk, Platform};
    use crate::{binance::retrieve_binance_order_book, hyperliquid::retrieve_hl_order_book};
    use approx::assert_relative_eq;

    #[tokio::test]
    async fn test_quote_binance() {
        let orderbook = retrieve_binance_order_book("ETHUSDT".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let quote = retrieve_quote(orderbook, 9_000_000.0).unwrap();

        println!("quote: {quote:#?}");
    }

    #[tokio::test]
    async fn test_quote_hl() {
        let orderbook = retrieve_hl_order_book("BTC".to_string(), BidAsk::Ask)
            .await
            .unwrap();

        let quote = retrieve_quote(orderbook, 1_000_000.0).unwrap();

        println!("quote: {quote:#?}");
    }

    #[test]
    fn test_buy_mock() -> Result<()> {
        let asks = get_mock_bids();

        // empty orderbook
        let quote = retrieve_quote(vec![], 246.0);
        assert!(quote.is_err());

        let quote = retrieve_quote(asks.clone(), 246.0);
        assert!(quote.is_err());

        let quote = retrieve_quote(asks, 245.0)?;
        assert_relative_eq!(quote.execution_price, 9.07, max_relative = 0.1);

        Ok(())
    }

    #[test]
    fn test_sell_mock() -> Result<()> {
        let asks = get_mock_asks();

        // empty orderbook
        let quote = retrieve_quote(vec![], 246.0);
        assert!(quote.is_err());

        // i want to sell more than all the asks combined
        let quote = retrieve_quote(asks.clone(), 246.0);
        assert!(quote.is_err());

        // first ask and half of the second one
        let quote = retrieve_quote(asks, 104.5)?;
        assert_relative_eq!(quote.execution_price, 8.36, max_relative = 0.1);

        Ok(())
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

    #[test]
    fn test_pct_diff() {
        let buying = calculate_pct_difference(102.0, 100.0); // buying
        let selling = calculate_pct_difference(100.0, 102.0); // selling
        assert_relative_eq!(buying, 0.02, max_relative = 0.001);
        println!("buying pct diff: {buying}");
        assert_relative_eq!(selling, 0.0196, max_relative = 0.001);
        println!("selling pct diff: {selling}");
    }

    fn get_mock_bids() -> Vec<LimitOrder> {
        vec![
            // 100.0
            LimitOrder {
                platform: Platform::Binance,
                price: 10.0,
                size: 10.0,
            },
            // 81.0
            LimitOrder {
                platform: Platform::Binance,
                price: 9.0,
                size: 9.0,
            },
            // 64.0
            LimitOrder {
                platform: Platform::Binance,
                price: 8.0,
                size: 8.0,
            },
        ]
    }

    fn get_mock_asks() -> Vec<LimitOrder> {
        vec![
            // 64.0
            LimitOrder {
                platform: Platform::Binance,
                price: 8.0,
                size: 8.0,
            },
            // 81.0
            LimitOrder {
                platform: Platform::Binance,
                price: 9.0,
                size: 9.0,
            },
            // 100.0
            LimitOrder {
                platform: Platform::Binance,
                price: 10.0,
                size: 10.0,
            },
        ]
    }
}
