use std::collections::VecDeque;

use crate::{
    games::{self, snake::snake},
    runner::{runner, RunnerHandle},
    ui::{self, handle_updates},
    updates::updates,
};

pub struct RuntimeState {
    pub input: Input,
    pub runner: RunnerHandle,
}

impl Default for RuntimeState {
    fn default() -> Self {
        let program = games::build(snake);

        let input = Input::default();
        let (updates_tx, updates_rx) = updates(&program);
        let (events_tx, runner) = runner(program, updates_tx);

        let set_program = ui::start(events_tx.clone());
        leptos::spawn_local(handle_updates(updates_rx, set_program));

        Self { input, runner }
    }
}

#[derive(Default)]
pub struct Input {
    pub buffer: VecDeque<u8>,
}
