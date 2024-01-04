mod script_loader;
mod watch;

use std::path::PathBuf;

use capi_core::repr::eval::fragments::FragmentId;
use crossbeam_channel::{Receiver, Sender};
use notify::RecommendedWatcher;
use notify_debouncer_mini::Debouncer;

use self::watch::watch;

pub struct Loader {
    watchers: Vec<Debouncer<RecommendedWatcher>>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
        }
    }

    /// Load the script at the given path
    ///
    /// # Implementation Note
    ///
    /// It would be better to not return the code as a `String` here, and
    /// instead trigger the first update through the channel. Then we only have
    /// to support one code path for loading code.
    pub fn load(
        &mut self,
        path: impl Into<PathBuf>,
        parent: Option<FragmentId>,
    ) -> anyhow::Result<(String, UpdateReceiver)> {
        let path = path.into();

        let (sender, receiver) = crossbeam_channel::unbounded();
        let watcher = watch(path, parent, sender)?;
        let (_, code) = receiver.recv()??;

        self.watchers.push(watcher);
        Ok((code, receiver))
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

pub type Update = anyhow::Result<(Option<FragmentId>, String)>;
pub type UpdateSender = Sender<Update>;
pub type UpdateReceiver = Receiver<Update>;
