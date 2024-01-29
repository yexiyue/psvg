use psvg::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_thread_ids(true).init();
    let dialogue = Cli::init()?;
    psvg::download::run(&dialogue).await?;
    Ok(())
}
