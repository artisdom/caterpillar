use std::{
    path::{Path, PathBuf},
    process,
};

use capi_watch::DebouncedChanges;
use tempfile::{tempdir, TempDir};
use tokio::{fs, process::Command, sync::watch, task};
use tracing::error;
use wasm_bindgen_cli_support::Bindgen;

pub fn start(changes: DebouncedChanges) -> UpdatesRx {
    let (tx, rx) = watch::channel(None);
    task::spawn(async {
        if let Err(err) = watch_and_build(changes, tx).await {
            error!("Build error: {err}");
            process::exit(1);
        }
    });
    rx
}

async fn watch_and_build(
    mut changes: DebouncedChanges,
    updates: UpdatesTx,
) -> anyhow::Result<()> {
    println!();
    println!("⏳ Starting initial build of Caterpillar...");
    println!();

    // We're not really doing anything with this variable, but it needs to
    // exist. It keeps the `TempDir` instances from being dropped before we're
    // done with it. Dropping it prematurely would delete the temporary
    // directory we serve files out of.
    let mut output_dir = None;

    build_once(&updates, &mut output_dir).await?;

    while changes.wait_for_change().await {
        println!();
        println!("🔄 Change detected.");
        println!("⏳ Rebuilding Caterpillar...");
        println!();

        let should_continue = build_once(&updates, &mut output_dir).await?;
        if let ShouldContinue::NoBecauseShutdown = should_continue {
            break;
        }
    }

    Ok(())
}

async fn build_once(
    updates: &UpdatesTx,
    output_dir: &mut Option<TempDir>,
) -> anyhow::Result<ShouldContinue> {
    for package in ["capi-runtime", "capi-debugger"] {
        let cargo_build = Command::new("cargo")
            .arg("build")
            .args(["--package", package])
            .args(["--target", "wasm32-unknown-unknown"])
            .status()
            .await?;
        if !cargo_build.success() {
            // The build failed, and since the rest of this function is
            // dependent on its success, we're done here.
            //
            // But that doesn't mean that the builder overall should be done.
            // Next time we detect a change, we should try again.
            return Ok(ShouldContinue::YesWhyNot);
        }
    }

    let target = "target/wasm32-unknown-unknown/debug";
    let new_output_dir = tempdir()?;
    copy(target, new_output_dir.path(), "capi-runtime.wasm").await?;

    let wasm_module = format!("{target}/capi-debugger.wasm");

    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(wasm_module)
        .web(true)?
        .generate(&new_output_dir)?;

    copy("capi", new_output_dir.path(), "index.html").await?;

    let output_path = new_output_dir.path().to_path_buf();

    if updates.send(Some(output_path)).is_err() {
        // If the send failed, the other end has hung up. That means either
        // we're currently shutting down, or something went wrong over there and
        // we _should_ be shutting down.
        return Ok(ShouldContinue::NoBecauseShutdown);
    }

    *output_dir = Some(new_output_dir);

    Ok(ShouldContinue::YesWhyNot)
}

async fn copy(
    source_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
    file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let file = file.as_ref();
    let source_dir = source_dir.as_ref();
    let target_dir = target_dir.as_ref();

    fs::copy(source_dir.join(file), target_dir.join(file)).await?;
    Ok(())
}

enum ShouldContinue {
    YesWhyNot,
    NoBecauseShutdown,
}

pub type UpdatesRx = watch::Receiver<Update>;
pub type UpdatesTx = watch::Sender<Update>;

pub type Update = Option<PathBuf>;
