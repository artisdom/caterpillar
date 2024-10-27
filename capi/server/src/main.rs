use std::path::PathBuf;

use capi_server::Event;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::parse();
    let mut events =
        capi_server::start(args.address.parse().unwrap(), args.serve_dir)
            .await?;

    while let Some(event) = events.recv().await {
        match event {
            Event::ChangeDetected => {
                println!("build:change");
            }
            Event::BuildFinished => {
                println!("build:finish");
            }
            Event::ServerReady => {
                println!("ready");
            }
        }
    }

    tracing::info!("`capi-server` shutting down.");
    Ok(())
}

/// Caterpillar server
#[derive(clap::Parser)]
pub struct Args {
    /// Address to serve at
    #[arg(short, long)]
    pub address: String,

    /// Directory to serve from
    #[arg(short, long)]
    pub serve_dir: PathBuf,
}

impl Args {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
