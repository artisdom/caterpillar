use std::{net::SocketAddr, path::PathBuf};

#[derive(clap::Parser)]
pub enum Args {
    Headless,
    Serve {
        /// Address to serve at
        #[arg(short, long, default_value = "127.0.0.1:34480")]
        address: SocketAddr,

        /// Directory to serve from
        #[arg(short, long)]
        serve_dir: PathBuf,
    },
}

impl Args {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
