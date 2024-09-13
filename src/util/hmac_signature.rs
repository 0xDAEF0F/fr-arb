use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn generate_hmac_signature(query_params: Option<String>) -> Result<String> {
    let binance_secret = std::env::var("BINANCE_SECRET_KEY")?;

    let mut mac = Hmac::<Sha256>::new_from_slice(binance_secret.as_bytes())?;

    if let Some(qp) = query_params {
        mac.update(qp.as_bytes())
    }

    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    Ok(signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corroborate_signature() -> Result<()> {
        let binance_secret = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        std::env::set_var("BINANCE_SECRET_KEY", binance_secret);

        let query_params = Some("timestamp=1578963600000".to_string());
        let signature = generate_hmac_signature(query_params)?;

        assert_eq!(
            signature,
            "d84e6641b1e328e7b418fff030caed655c266299c9355e36ce801ed14631eed4"
        );

        Ok(())
    }
}
