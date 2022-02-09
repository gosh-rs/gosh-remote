// [[file:../../remote.note::45c2eed2][45c2eed2]]
use gosh_remote::cli::*;

#[tokio::main]
async fn main() -> Result<()> {
    remote_enter_main().await?;
    log_dbg!();

    Ok(())
}
// 45c2eed2 ends here
