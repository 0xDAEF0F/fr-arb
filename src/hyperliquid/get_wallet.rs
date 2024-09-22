use anyhow::Result;
use ethers::signers::{coins_bip39::English, LocalWallet, MnemonicBuilder};

pub fn get_wallet() -> Result<LocalWallet> {
    let mnemonic = std::env::var("MNEMONIC")?;

    let wallet: LocalWallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic.as_str())
        .build()?;

    Ok(wallet)
}
