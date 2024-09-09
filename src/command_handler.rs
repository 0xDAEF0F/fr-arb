use crate::balances::{build_account_balance_table, build_account_open_positions_table};
use anyhow::Result;
use std::io::Write;

pub async fn handle_command(line: String, stdout: &mut impl Write) -> Result<()> {
    match line.as_str() {
        "balance" => {
            let global_balances = build_account_balance_table().await?;
            let open_positions = build_account_open_positions_table().await?;
            writeln!(stdout, "{}", global_balances)?;
            writeln!(stdout, "{}", open_positions)?;
        }
        _ => {}
    }
    Ok(())
}
