use crate::binance::retrieve_binance_order_book;
use crate::compare_funding_rates::compare_funding_rates;
use crate::constants::BINANCE_FEE;
use crate::constants::HYPERLIQUID_FEE;
use crate::hyperliquid::retrieve_hl_order_book;
use crate::util::determine_short_based_on_fr;
use crate::util::BidAsk;
use crate::util::{Orderbook, Platform};
use anyhow::bail;
use anyhow::Result;

#[derive(Debug)]
pub struct Quote {
    pub platform: Platform,
    pub expected_execution_price: f64,
    pub platform_fees: f64, // decimal pct
    pub slippage: f64,      // decimal pct
    pub size: f64,
}

/// first quote represents sell/short
pub async fn retrieve_quote(token: String, amt: f64) -> Result<(Quote, Quote)> {
    let jfr = compare_funding_rates()
        .await?
        .into_iter()
        .find(|jfr| jfr.name == token)
        .expect("token must be in joint funding rates");

    let platform = determine_short_based_on_fr(jfr);

    let (short_orderbook, long_orderbook) = match platform {
        Platform::Binance => (
            retrieve_binance_order_book(token.clone(), BidAsk::Bid).await?,
            retrieve_hl_order_book(token.clone(), BidAsk::Ask).await?,
        ),
        Platform::Hyperliquid => (
            retrieve_hl_order_book(token.clone(), BidAsk::Bid).await?,
            retrieve_binance_order_book(token.clone(), BidAsk::Ask).await?,
        ),
    };

    let quote_a = retrieve_quote_(short_orderbook, amt / 2.0)?;
    let quote_b = retrieve_quote_(long_orderbook, amt / 2.0)?;

    Ok((quote_a, quote_b))
}

fn retrieve_quote_(orderbook: Orderbook, amount: f64) -> Result<Quote> {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    if orderbook.limit_orders.is_empty() {
        bail!("empty orderbook")
    }

    let first_price = orderbook.limit_orders[0].price;
    let platform = orderbook.platform;
    for bid_ask in orderbook.limit_orders {
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
    // leave one dollar as dust (more than enough)
    if remaining_amount > 1.0 {
        bail!("orderbook can't cover the amount to buy/sell")
    }

    let execution_price = total_cost / total_quantity;

    let quote = Quote {
        expected_execution_price: execution_price,
        platform,
        slippage: calculate_pct_difference(execution_price, first_price),
        size: total_quantity,
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
    use crate::util::{LimitOrder, Platform};
    use approx::assert_relative_eq;

    #[test]
    fn test_sell_scenario() -> Result<()> {
        let bids = get_mock_bids();

        // empty orderbook
        let quote = retrieve_quote_(
            Orderbook {
                platform: Platform::Binance,
                limit_orders: vec![],
            },
            246.0,
        );
        assert!(quote.is_err());

        let quote = retrieve_quote_(bids.clone(), 246.0);
        assert!(quote.is_err());

        let quote = retrieve_quote_(bids, 245.0)?;
        assert_relative_eq!(quote.expected_execution_price, 9.07, max_relative = 0.1);

        Ok(())
    }

    #[test]
    fn test_buy_scenario() -> Result<()> {
        let asks = get_mock_asks();

        // empty orderbook
        let quote = retrieve_quote_(
            Orderbook {
                platform: Platform::Binance,
                limit_orders: vec![],
            },
            246.0,
        );
        assert!(quote.is_err());

        // i want to sell more than all the asks combined
        let quote = retrieve_quote_(asks.clone(), 246.0);
        assert!(quote.is_err());

        // first ask and half of the second one
        let quote = retrieve_quote_(asks, 104.5)?;
        assert_relative_eq!(quote.expected_execution_price, 8.36, max_relative = 0.1);

        Ok(())
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

    fn get_mock_bids() -> Orderbook {
        Orderbook {
            platform: Platform::Binance,
            limit_orders: vec![
                LimitOrder {
                    price: 10.0,
                    size: 10.0,
                },
                LimitOrder {
                    price: 9.0,
                    size: 9.0,
                },
                LimitOrder {
                    price: 8.0,
                    size: 8.0,
                },
            ],
        }
    }

    fn get_mock_asks() -> Orderbook {
        Orderbook {
            platform: Platform::Binance,
            limit_orders: vec![
                LimitOrder {
                    price: 8.0,
                    size: 8.0,
                },
                LimitOrder {
                    price: 9.0,
                    size: 9.0,
                },
                LimitOrder {
                    price: 10.0,
                    size: 10.0,
                },
            ],
        }
    }
}
