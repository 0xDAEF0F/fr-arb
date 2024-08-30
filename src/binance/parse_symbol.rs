pub struct Pair {
    pub base: String,
    pub quote: String,
}

pub fn parse_symbol(symbol: String) -> Option<Pair> {
    let quote_currencies = ["USDT", "USDC"];

    for quote in quote_currencies.iter() {
        if symbol.ends_with(quote) {
            let base = symbol[..symbol.len() - quote.len()].to_string();
            return Some(Pair {
                base,
                quote: quote.to_string(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_symbol_with_usdt() {
        let symbol = "BTCUSDT".to_string();
        let result = parse_symbol(symbol);
        assert!(result.is_some());
        let pair = result.unwrap();
        assert_eq!(pair.base, "BTC");
        assert_eq!(pair.quote, "USDT");
    }

    #[test]
    fn test_parse_symbol_with_usdc() {
        let symbol = "ETHUSDC".to_string();
        let result = parse_symbol(symbol);
        assert!(result.is_some());
        let pair = result.unwrap();
        assert_eq!(pair.base, "ETH");
        assert_eq!(pair.quote, "USDC");
    }

    #[test]
    fn test_parse_symbol_with_unsupported_quote() {
        let symbol = "BTCBUSD".to_string();
        let result = parse_symbol(symbol);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_symbol_with_invalid_format() {
        let symbol = "INVALIDFORMAT".to_string();
        let result = parse_symbol(symbol);
        assert!(result.is_none());
    }
}
