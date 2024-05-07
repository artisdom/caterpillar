use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
};

use capi_runtime::{
    DebugEvent, Effect, Program, ProgramEffect, ProgramState, Value,
};

use crate::{display::TILES_PER_AXIS, server::EventsRx, updates::UpdatesTx};

pub struct RunnerThread {
    effects: EffectsRx,
    resume: ResumeTx,
}

impl RunnerThread {
    pub fn start(
        program: Program,
        events: EventsRx,
        updates: UpdatesTx,
    ) -> Self {
        let (effects_tx, effects_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();

        let mut runner = Runner {
            program,
            events,
            updates,
            effects: effects_tx,
            resume: resume_rx,
        };

        thread::spawn(move || {
            runner.start();
        });

        Self {
            effects: effects_rx,
            resume: resume_tx,
        }
    }

    pub fn effects(&mut self) -> impl Iterator<Item = DisplayEffect> + '_ {
        self.effects.try_iter()
    }

    pub fn resume(&mut self) {
        // This will cause an error, if the other end has hung up, which happens
        // if the program has ended. Nothing we can do about it.
        let _ = self.resume.send(());
    }
}

struct Runner {
    program: Program,
    events: EventsRx,
    updates: UpdatesTx,
    effects: EffectsTx,
    resume: ResumeRx,
}

impl Runner {
    fn start(&mut self) {
        self.program.push(ARGUMENTS);

        loop {
            self.updates.send(&self.program);

            let mut event = if self.program.state.is_running() {
                None
            } else {
                // If we're not running, the program won't step anyway, and
                // there's no point in busy-looping while nothing changes.
                //
                // Just wait until we receive an event from the client.
                Some(self.events.blocking_recv().unwrap())
            };

            while let Some(event) =
                event.take().or_else(|| self.events.try_recv().ok())
            {
                // This doesn't work so well. This receive loop was moved here,
                // so we can have some control over the program from the
                // debugger, while it is stuck in an endless loop.
                //
                // And this works somewhat. We can send events. But unless those
                // events result in the program to stop running, we won't see
                // any indication of them being received in the debugger, as the
                // program isn't sent when it's running.

                match event {
                    DebugEvent::Reset => {
                        self.program.reset();
                        self.program.push(ARGUMENTS);
                    }
                    DebugEvent::ToggleBreakpoint { address } => {
                        self.program.toggle_breakpoint(address);
                    }
                }
            }

            self.program.step();
            if let ProgramState::Effect {
                effect: ProgramEffect::Builtin(effect),
                ..
            } = &self.program.state
            {
                match effect {
                    Effect::Error(_) => {}
                    Effect::SetTile { x, y, value } => {
                        let x = *x;
                        let y = *y;
                        let value = *value;
                        self.effects
                            .send(DisplayEffect::SetTile { x, y, value })
                            .unwrap();

                        self.program.state = ProgramState::Running;
                    }
                    Effect::RequestRedraw => {
                        self.program.state = ProgramState::Running;

                        self.effects.send(DisplayEffect::SubmitTiles).unwrap();

                        // The purpose of the "submit tiles" effect is to serve
                        // as a synchronization point, where the program can
                        // pause until the display code has processed the effect
                        // (and all leading up to it).
                        //
                        // Once this is a dedicated thread, we need to wait here
                        // until a signal is received on the `resume` channel.
                        // But until then, we can't do that.
                        assert_eq!(
                            self.resume.try_recv(),
                            Err(TryRecvError::Empty)
                        );
                    }
                }
            }
        }
    }
}

type EffectsTx = mpsc::Sender<DisplayEffect>;
type EffectsRx = mpsc::Receiver<DisplayEffect>;

type ResumeTx = mpsc::Sender<()>;
type ResumeRx = mpsc::Receiver<()>;

#[derive(Debug)]
pub enum DisplayEffect {
    SetTile { x: u8, y: u8, value: u8 },
    SubmitTiles,
}

// I don't like the `as` here, but `.try_into().unwrap()` doesn't work in a
// const context.
const ARGUMENTS: [Value; 2] = [Value(TILES_PER_AXIS as u8); 2];
