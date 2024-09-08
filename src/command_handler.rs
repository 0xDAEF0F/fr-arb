use anyhow::Result;
use std::io::Write;

pub fn handle_command(line: String, stdout: &mut impl Write) -> Result<()> {
    match line.as_str() {
        "balance" => {
            writeln!(stdout, "Balance is 50")?;
        }
        _ => {}
    }
    Ok(())
}
