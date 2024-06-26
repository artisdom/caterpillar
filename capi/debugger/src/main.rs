pub mod ffi;
pub mod model;
pub mod remote_process;
pub mod state;
pub mod ui;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Error)
        .expect("Failed to initialize logging to console");
}
