use crate::balances::build_account_balance_table;
use anyhow::Result;
use std::io::Write;

pub async fn handle_command(line: String, stdout: &mut impl Write) -> Result<()> {
    match line.as_str() {
        "balance" => {
            let formatted_table = build_account_balance_table().await?;
            writeln!(stdout, "{}", formatted_table)?;
        }
        _ => {}
    }
    Ok(())
}
