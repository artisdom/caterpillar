pub mod command;
pub mod runtime_state;
pub mod updates;

/// The size of the updates buffer
///
/// This is a ridiculous 1 MiB large. It should be possible to make this much
/// smaller, but for now, we're using a very space-inefficient serialization
/// format.
pub const UPDATES_BUFFER_SIZE: usize = 1024 * 1024;

/// The size of the commands buffer
///
/// This is a ridiculous 1 MiB large. It should be possible to make this much
/// smaller, but for now, we're using a very space-inefficient serialization
/// format.
pub const COMMANDS_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Versioned<T> {
    pub timestamp: u64,
    pub inner: T,
}
