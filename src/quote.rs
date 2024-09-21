use crate::binance::retrieve_binance_order_book;
use crate::compare_funding_rates::compare_funding_rates;
use crate::constants::{BINANCE_FEE, HYPERLIQUID_FEE};
use crate::hyperliquid::retrieve_hl_order_book;
use crate::util::{calculate_pct_difference, determine_short_based_on_fr, LimitOrder, Platform};
use anyhow::{bail, Result};
use tokio::try_join;

#[derive(Debug)]
pub struct Quote {
    pub platform: Platform,
    pub expected_execution_price: f64,
    pub mid_price: f64,
    pub platform_fees: f64, // decimal pct
    pub slippage: f64,      // decimal pct
    pub size: f64,
}

/// first quote represents sell/short
pub async fn retrieve_quote_enter(token: String, amt: f64) -> Result<(Quote, Quote)> {
    let jfr = compare_funding_rates()
        .await?
        .into_iter()
        .find(|jfr| jfr.name == token)
        .expect("token must be in joint funding rates");

    let platform = determine_short_based_on_fr(jfr);

    let (short_orderbook, long_orderbook) = match platform {
        Platform::Binance => try_join!(
            retrieve_binance_order_book(token.clone()),
            retrieve_hl_order_book(token.clone()),
        )?,
        Platform::Hyperliquid => try_join!(
            retrieve_hl_order_book(token.clone()),
            retrieve_binance_order_book(token.clone()),
        )?,
    };

    let mid_price_a = (short_orderbook.bids[0].price + short_orderbook.asks[0].price) / 2.0;
    let quote_a = retrieve_quote_(
        short_orderbook.bids,
        amt / 2.0,
        mid_price_a,
        short_orderbook.platform,
    )?;

    let mid_price_b = (long_orderbook.bids[0].price + long_orderbook.asks[0].price) / 2.0;
    let quote_b = retrieve_quote_(
        long_orderbook.asks,
        amt / 2.0,
        mid_price_b,
        long_orderbook.platform,
    )?;

    Ok((quote_a, quote_b))
}

pub fn retrieve_quote_(
    orderbook: Vec<LimitOrder>,
    amount: f64,
    mid_price: f64,
    platform: Platform,
) -> Result<Quote> {
    let mut remaining_amount = amount;
    let mut total_cost = 0.0;
    let mut total_quantity = 0.0;

    if orderbook.is_empty() {
        bail!("empty orderbook")
    }

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
    // leave one dollar as dust (more than enough)
    if remaining_amount > 1.0 {
        bail!("orderbook can't cover the amount to buy/sell")
    }

    let execution_price = total_cost / total_quantity;

    let quote = Quote {
        expected_execution_price: execution_price,
        mid_price,
        platform,
        slippage: calculate_pct_difference(execution_price, mid_price),
        size: total_quantity,
        platform_fees: match platform {
            Platform::Binance => BINANCE_FEE,
            Platform::Hyperliquid => HYPERLIQUID_FEE,
        },
    };

    Ok(quote)
}

pub fn get_expected_execution_price(orderbook: Vec<LimitOrder>, size: f64) -> Result<f64> {
    if orderbook.is_empty() {
        bail!("empty orderbook")
    }

    let mut remaining_size = size;
    let mut total_cost = 0.0;

    for order in orderbook {
        if order.size >= remaining_size {
            total_cost += remaining_size * order.price;
            remaining_size = 0.0;
            break;
        }

        remaining_size -= order.size;
        total_cost += order.size * order.price;
    }

    if remaining_size > 0.0 {
        bail!("orderbook can't cover the requested size")
    }

    Ok(total_cost / size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{LimitOrder, Platform};
    use approx::assert_relative_eq;

    #[test]
    fn test_get_expected_execution_price() -> Result<()> {
        let orderbook = vec![
            LimitOrder {
                price: 100.0,
                size: 10.0,
            },
            LimitOrder {
                price: 101.0,
                size: 15.0,
            },
            LimitOrder {
                price: 102.0,
                size: 20.0,
            },
        ];
        let result = get_expected_execution_price(orderbook, 25.0)?;
        assert_relative_eq!(result, 100.6, max_relative = 0.0);

        // Test case 3: Partial fill of last order
        let orderbook = vec![
            LimitOrder {
                price: 100.0,
                size: 10.0,
            },
            LimitOrder {
                price: 101.0,
                size: 15.0,
            },
            LimitOrder {
                price: 102.0,
                size: 20.0,
            },
        ];
        let result = get_expected_execution_price(orderbook, 30.0)?;
        assert_relative_eq!(result, 100.8333333, epsilon = 1e-6);

        let orderbook = vec![];
        assert!(get_expected_execution_price(orderbook, 10.0).is_err());

        let orderbook = vec![
            LimitOrder {
                price: 100.0,
                size: 10.0,
            },
            LimitOrder {
                price: 101.0,
                size: 15.0,
            },
        ];
        assert!(get_expected_execution_price(orderbook, 30.0).is_err());

        Ok(())
    }

    #[test]
    fn test_sell_scenario() -> Result<()> {
        let bids = get_mock_bids();

        // empty orderbook
        let quote = retrieve_quote_(vec![], 247.0, 0.0, Platform::Binance);
        assert!(quote.is_err());

        let quote = retrieve_quote_(bids.clone(), 247.0, 0.0, Platform::Binance);
        assert!(quote.is_err());

        let quote = retrieve_quote_(bids, 245.0, 0.0, Platform::Binance)?;
        assert_relative_eq!(quote.expected_execution_price, 9.07, max_relative = 0.1);

        Ok(())
    }

    #[test]
    fn test_buy_scenario() -> Result<()> {
        let asks = get_mock_asks();

        // empty orderbook
        let quote = retrieve_quote_(vec![], 247.0, 0.0, Platform::Hyperliquid);
        assert!(quote.is_err());

        // buy more than orderbook depth
        let quote = retrieve_quote_(asks.clone(), 247.0, 0.0, Platform::Hyperliquid);
        assert!(quote.is_err());

        // first ask and half of the second one
        let quote = retrieve_quote_(asks, 104.5, 0.0, Platform::Hyperliquid)?;
        assert_relative_eq!(quote.expected_execution_price, 8.36, max_relative = 0.1);

        Ok(())
    }

    fn get_mock_bids() -> Vec<LimitOrder> {
        vec![
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
        ]
    }

    fn get_mock_asks() -> Vec<LimitOrder> {
        vec![
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
        ]
    }
}
