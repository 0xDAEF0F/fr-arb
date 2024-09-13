mod balances;
mod binance;
mod command_handler;
mod compare_funding_rates;
mod constants;
mod funding_history;
mod hyperliquid;
mod quote;
mod util;

use anyhow::Result;
use command_handler::handle_command;
use rustyline_async::{Readline, ReadlineEvent};
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let (mut rl, mut stdout) = Readline::new("> ".into())?;

    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(60)) => {
                writeln!(stdout, "message received")?;
            }
            cmd = rl.readline() => match cmd {
                Ok(ReadlineEvent::Line(line)) => {
                   handle_command(line, &mut rl, &mut stdout).await?;
                }
                Ok(ReadlineEvent::Eof) => {
                    writeln!(stdout, "<EOF>")?;
                    break;
                }
                Ok(ReadlineEvent::Interrupted) => {
                    writeln!(stdout, "^C")?;
                    break;
                }
                Err(e) => {
                    writeln!(stdout, "Error: {e:?}")?;
                    break;
                }
            }
        }
    }
    rl.flush()?;

    Ok(())
}
