use std::collections::VecDeque;

use rand::random;
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    display::Display,
    effects::EffectsRx,
    ffi,
    games::{self, snake::snake},
    program::{ProgramEffect, ProgramEffectKind},
    runtime::{BuiltinEffect, EvaluatorEffectKind, Value},
    tiles::NUM_TILES,
    ui,
    updates::updates,
};

pub struct RuntimeState {
    pub input: Input,
    pub on_frame: mpsc::UnboundedSender<()>,
    pub effects_rx: EffectsRx,
    pub tiles: [u8; NUM_TILES],
    pub display: Option<Display>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        let mut program = games::build(snake);

        let input = Input::default();
        let (on_frame_tx, mut on_frame_rx) = mpsc::unbounded_channel();
        let (mut updates_tx, updates_rx) = updates(&program);
        let (events_tx, mut events_rx) = mpsc::unbounded_channel();
        let (_, effects_rx) = mpsc::unbounded_channel();

        leptos::spawn_local(async move {
            loop {
                loop {
                    match events_rx.try_recv() {
                        Ok(event) => {
                            program.process_event(event);
                        }
                        Err(TryRecvError::Empty) => {
                            break;
                        }
                        Err(TryRecvError::Disconnected) => {
                            // The other end has hung up, which happens during
                            // shutdown. Shut down this task, too.
                            return;
                        }
                    }
                }

                if !program.can_step() {
                    // If the program won't step anyway, then there's no point
                    // in busy-looping while nothing changes.
                    //
                    // Just wait until we receive an event from the client.
                    let event = events_rx.recv().await.unwrap();
                    program.process_event(event);
                }

                program.step();

                if let Some(ProgramEffect {
                    kind:
                        ProgramEffectKind::Evaluator(EvaluatorEffectKind::Builtin(
                            effect,
                        )),
                    ..
                }) = program.effects.front()
                {
                    match effect {
                        BuiltinEffect::Error(_) => {
                            // Nothing needs to be done. With an unhandled
                            // effect, the program won't continue running, and
                            // the debugger will see the error and display it.
                        }
                        BuiltinEffect::Load { address } => {
                            let address: usize = (*address).into();
                            let value = program.memory.inner[address];
                            program.push([value]);

                            program.effects.pop_front();
                        }
                        BuiltinEffect::Store { address, value } => {
                            let address: usize = (*address).into();
                            program.memory.inner[address] = *value;

                            program.effects.pop_front();
                        }
                        BuiltinEffect::SetTile { x, y, value } => {
                            let x = *x;
                            let y = *y;
                            let value = *value;

                            program.effects.pop_front();

                            let mut state = ffi::STATE.inner.lock().unwrap();
                            let state =
                                state.get_or_insert_with(Default::default);

                            let Some(display) = state.display.as_mut() else {
                                // Display not initialized yet.
                                continue;
                            };

                            display.set_tile(
                                x.into(),
                                y.into(),
                                value,
                                &mut state.tiles,
                            );
                        }
                        BuiltinEffect::SubmitFrame => {
                            // This effect serves as a synchronization point
                            // between the program and the display code. Before
                            // we continue running, we need to wait here, until
                            // the display code has confirmed that we're ready
                            // to continue.
                            let () = on_frame_rx.recv().await.unwrap();

                            program.effects.pop_front();
                        }
                        BuiltinEffect::ReadInput => {
                            let mut state = ffi::STATE.inner.lock().unwrap();
                            let state =
                                state.get_or_insert_with(Default::default);

                            let input = state
                                .input
                                .buffer
                                .pop_front()
                                .unwrap_or(0)
                                .try_into()
                                .unwrap();

                            program.push([Value(input)]);
                            program.effects.pop_front();
                        }
                        BuiltinEffect::ReadRandom => {
                            program.push([Value(random())]);
                            program.effects.pop_front();
                        }
                    }
                }

                updates_tx.send_if_relevant_change(&program);
            }
        });

        ui::start(updates_rx, events_tx);

        // While we're still using `pixels`, the `Display` constructor needs to
        // be async. We need to do some acrobatics here to deal with that.
        leptos::spawn_local(async {
            let display = Display::new().await.unwrap();

            let mut state = ffi::STATE.inner.lock().unwrap();
            let state = state.get_or_insert_with(Default::default);

            state.display = Some(display);
        });

        Self {
            input,
            on_frame: on_frame_tx,
            effects_rx,
            tiles: [0; NUM_TILES],
            display: None,
        }
    }
}

#[derive(Default)]
pub struct Input {
    pub buffer: VecDeque<u8>,
}
