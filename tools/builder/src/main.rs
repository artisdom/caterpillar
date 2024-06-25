mod build;
mod pipeline;
mod serve;
mod watch;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    pipeline::pipeline().await?;

    Ok(())
}
