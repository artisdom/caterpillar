use std::panic;

use capi_game_engine::{
    display,
    game_engine::GameEngine,
    host::{GameEngineEffect, GameEngineHost},
};
use capi_process::{CoreEffect, Effect, Value};
use capi_protocol::{
    command::{Command, SerializedCommand},
    updates::Updates,
};

use crate::ffi_out::on_panic;

pub struct RuntimeState {
    pub game_engine: GameEngine,
    pub commands: Vec<SerializedCommand>,
    pub updates: Updates<GameEngineHost>,
}

impl RuntimeState {
    pub fn new() -> Self {
        panic::set_hook(Box::new(|panic_info| {
            on_panic(&panic_info.to_string());
        }));

        Self {
            game_engine: GameEngine::new(),
            commands: Vec::new(),
            updates: Updates::default(),
        }
    }

    pub fn update(&mut self, pixels: &mut [u8]) {
        for command in self.commands.drain(..) {
            let command = Command::deserialize(command);

            match command {
                Command::BreakpointClear { instruction } => {
                    self.game_engine
                        .process
                        .clear_durable_breakpoint(&instruction);
                }
                Command::BreakpointSet { instruction } => {
                    self.game_engine
                        .process
                        .set_durable_breakpoint(instruction);
                }
                Command::Continue { and_stop_at } => {
                    self.game_engine.process.continue_(and_stop_at);
                }
                Command::Reset => self.game_engine.reset(),
                Command::Step => {
                    if let Some(Effect::Core(CoreEffect::Breakpoint)) = self
                        .game_engine
                        .process
                        .state()
                        .first_unhandled_effect()
                    {
                        let and_stop_at =
                            self.game_engine.process.stack().next_instruction();
                        self.game_engine.process.continue_(Some(and_stop_at))
                    } else {
                        // If we're not stopped at a breakpoint, we can't step.
                        // It would be better, if this resulted in an explicit
                        // error that is sent to the debugger, instead of
                        // silently being ignored here.
                    }
                }
                Command::Stop => {
                    self.game_engine.process.stop();
                }
            }
        }

        let Some(bytecode) = &self.game_engine.bytecode else {
            return;
        };

        while self.game_engine.process.state().can_step() {
            self.game_engine.process.step(bytecode);

            if let Some(effect) =
                self.game_engine.process.state().first_unhandled_effect()
            {
                match effect {
                    Effect::Core(CoreEffect::Breakpoint) => {
                        // Nothing to do here. With an unhandled effect, the
                        // program won't continue running. The debugger is in
                        // control of what happens next.
                    }
                    Effect::Host(GameEngineEffect::Load { address }) => {
                        let address: usize = (*address).into();
                        let value = self.game_engine.memory.inner[address];
                        let value: i32 = value.into();
                        self.game_engine
                            .process
                            .push([Value(value.to_le_bytes())]);

                        self.game_engine.process.handle_first_effect();
                    }
                    Effect::Host(GameEngineEffect::Store {
                        address,
                        value,
                    }) => {
                        let address: usize = (*address).into();
                        self.game_engine.memory.inner[address] = *value;

                        self.game_engine.process.handle_first_effect();
                    }
                    Effect::Host(GameEngineEffect::SetTile { x, y, color }) => {
                        let x = *x;
                        let y = *y;
                        let color = *color;

                        self.game_engine.process.handle_first_effect();

                        display::set_tile(x.into(), y.into(), color, pixels);
                    }
                    Effect::Host(GameEngineEffect::SubmitFrame) => {
                        // This effect means that the game is done rendering.
                        // Let's break out of this loop now, so we can do our
                        // part in that and return control to the host.
                        self.game_engine.process.handle_first_effect();
                        break;
                    }
                    Effect::Host(GameEngineEffect::ReadInput) => {
                        let input: i32 = self
                            .game_engine
                            .input
                            .buffer
                            .pop_front()
                            .unwrap_or(0)
                            .into();

                        self.game_engine
                            .process
                            .push([Value(input.to_le_bytes())]);
                        self.game_engine.process.handle_first_effect();
                    }
                    Effect::Host(GameEngineEffect::ReadRandom) => {
                        // We get a lot of random numbers from the host, and
                        // they are topped off every frame. It should be a
                        // while, before Caterpillar programs become complex
                        // enough to run into this limit.
                        //
                        // If that happens, and we hit this `unwrap`, we can of
                        // course just increase the limit. But long-term, it
                        // probably makes more sense to implement a PRNG, either
                        // in Rust or Caterpillar, and only seed that with
                        // randomness from the host.
                        let random =
                            self.game_engine.random.pop_front().unwrap();

                        self.game_engine
                            .process
                            .push([Value(random.to_le_bytes())]);
                        self.game_engine.process.handle_first_effect();
                    }
                    _ => {
                        // Nothing needs to be done. With an unhandled
                        // effect, the program won't continue running, and
                        // the debugger will see the error and display it.
                    }
                }
            }
        }

        self.updates
            .queue_updates(&self.game_engine.process, &self.game_engine.memory);
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}
