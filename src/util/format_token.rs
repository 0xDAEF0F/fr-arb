/// first element in tuple is for binance
pub fn format_token(t: &str) -> (String, String) {
    match t {
        "PEPE" | "FLOKI" | "BONK" => (format!("1000{t}"), format!("k{t}")),
        _ => (t.to_string(), t.to_string()),
    }
}
