use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify_debouncer_mini::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEventKind, Debouncer,
};
use tempfile::tempdir;
use tokio::{
    fs,
    process::Command,
    sync::watch,
    task::{self},
};

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let serve_dir = tempdir()?;

    let _host_watcher = watch_host(serve_dir.path().to_path_buf())?;
    let _runtime_watcher = watch_runtime(serve_dir.path().to_path_buf())?;
    serve(&serve_dir).await?;

    Ok(())
}

fn watch_host(
    serve_dir: PathBuf,
) -> anyhow::Result<Debouncer<RecommendedWatcher>> {
    let (tx, rx) = watch::channel(());
    tx.send_replace(());

    let mut debouncer = new_debouncer(
        Duration::from_millis(50),
        move |result: DebounceEventResult| {
            let events = result.unwrap();
            for event in events {
                if event.kind == DebouncedEventKind::Any {
                    tx.send(()).unwrap();
                }
            }
        },
    )?;
    debouncer
        .watcher()
        .watch(Path::new("index.html"), RecursiveMode::Recursive)?;

    task::spawn(copy_host(serve_dir, rx));

    Ok(debouncer)
}

async fn copy_host(
    serve_dir: PathBuf,
    mut changes: watch::Receiver<()>,
) -> anyhow::Result<()> {
    loop {
        let () = changes.changed().await.unwrap();

        println!("Copying host...");
        copy_artifacts(&serve_dir).await?;
    }
}

fn watch_runtime(
    serve_dir: PathBuf,
) -> anyhow::Result<Debouncer<RecommendedWatcher>> {
    let (tx, rx) = watch::channel(());
    tx.send_replace(());

    let mut debouncer = new_debouncer(
        Duration::from_millis(50),
        move |result: DebounceEventResult| {
            let events = result.unwrap();
            for event in events {
                if event.kind == DebouncedEventKind::Any {
                    tx.send(()).unwrap();
                }
            }
        },
    )?;
    debouncer
        .watcher()
        .watch(Path::new("capi-runtime"), RecursiveMode::Recursive)?;

    task::spawn(build_runtime(serve_dir, rx));

    Ok(debouncer)
}

async fn build_runtime(
    serve_dir: impl AsRef<Path>,
    mut changes: watch::Receiver<()>,
) -> anyhow::Result<()> {
    let serve_dir = serve_dir.as_ref();

    loop {
        let () = changes.changed().await.unwrap();

        // Remove all files before the build, to prevent anybody from loading a
        // stale version after a change.
        let mut read_dir = fs::read_dir(serve_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            fs::remove_file(entry.path()).await?;
        }

        let exit_status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .args(["--package", "capi-runtime"])
            .args(["--target", "wasm32-unknown-unknown"])
            .status()
            .await?;

        if exit_status.success() {
            copy_artifacts(serve_dir).await?;
        }
    }
}

async fn copy_artifacts(serve_dir: &Path) -> anyhow::Result<()> {
    fs::copy("index.html", serve_dir.join("index.html")).await?;
    fs::copy(
        "target/wasm32-unknown-unknown/release/capi_runtime.wasm",
        serve_dir.join("capi_runtime.wasm"),
    )
    .await?;

    Ok(())
}

async fn serve(serve_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    rocket::build()
        .mount("/", rocket::fs::FileServer::from(&serve_dir))
        .mount("/", rocket::routes![code])
        .launch()
        .await?;

    Ok(())
}

#[rocket::get("/code")]
fn code() -> &'static [u8] {
    &[
        0x04, // clone
        0x01, // push
        0,    // address
        0x03, // store
        0x04, // clone
        0x01, // push
        1,    // address
        0x03, // store
        0x01, // push
        2,    // address
        0x03, // store
        0x01, // push
        255,  // alpha channel
        0x01, // push
        3,    // address
        0x03, // store
        0x00, // terminate
    ]
}
