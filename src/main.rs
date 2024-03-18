use std::{path::Path, time::Duration};

use notify_debouncer_mini::DebounceEventResult;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut debouncer = notify_debouncer_mini::new_debouncer(
        Duration::from_millis(50),
        |result: DebounceEventResult| {
            let events = result.expect("Error watching for changes");
            dbg!(events);
        },
    )?;
    debouncer
        .watcher()
        .watch(Path::new("index.html"), notify::RecursiveMode::NonRecursive)?;

    let serve_dir = tempfile::tempdir()?;
    fs::copy("index.html", serve_dir.path().join("index.html")).await?;

    warp::serve(warp::fs::dir(serve_dir.path().to_owned()))
        .run(([127, 0, 0, 1], 8080))
        .await;

    Ok(())
}
