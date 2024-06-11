mod breakpoints;
mod code;
mod compiler;
mod debugger;
mod display;
mod effects;
mod ffi;
mod games;
mod program;
mod runner;
mod runtime;
mod source_map;
mod state;
mod syntax;
mod ui;
mod updates;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Error)
        .expect("Failed to initialize logging to console");

    leptos::spawn_local(main_async());
}

async fn main_async() {
    crate::display::run().await.unwrap();
}
