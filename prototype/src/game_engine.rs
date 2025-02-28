use crate::{
    actor::{Actor, Sender, ThreadHandle},
    editor::Editor,
    language::{
        host::Host,
        interpreter::{Interpreter, InterpreterState},
    },
};

pub struct GameEngine {
    pub threads: GameEngineThreads,
    pub senders: GameEngineSenders,
}

impl GameEngine {
    pub fn start(game_output_tx: Sender<GameOutput>) -> anyhow::Result<Self> {
        let host = Host::empty();
        let mut editor = Editor::default();
        let mut interpreter = Interpreter::new(editor.code());

        editor.render(&host, &interpreter)?;

        let handle_events = Actor::spawn(move |event| {
            match event {
                Event::EditorInput { line } => {
                    editor.process_input(line, &host, &mut interpreter);

                    match interpreter.step(editor.code()) {
                        InterpreterState::CallToHostFunction {
                            id: _,
                            input: _,
                            output: _,
                        } => {
                            // No host functions are defined, currently.
                        }
                        InterpreterState::Error => {
                            // Not handling errors right now. Eventually, those
                            // should be properly encoded in `Code` and
                            // therefore visible in the editor. But in any case,
                            // there's nothing to do here, at least for now.
                        }
                        InterpreterState::Finished { output } => {
                            let color = output as f64 / 255.;

                            game_output_tx.send(GameOutput::SubmitColor {
                                color: [color, color, color, 1.],
                            })?;
                        }
                    }

                    editor.render(&host, &interpreter)?;
                }
                Event::GameInput(GameInput::RenderingFrame) => {
                    // This loop is coupled to the frame rate of the renderer.
                }
            }

            Ok(())
        });

        let events_from_editor_input = handle_events.sender.clone();
        let handle_editor_input = Actor::spawn(move |line| {
            events_from_editor_input.send(Event::EditorInput { line })?;
            Ok(())
        });

        let events_from_game_input = handle_events.sender;
        let handle_game_input = Actor::spawn(move |input| {
            events_from_game_input.send(Event::GameInput(input))?;
            Ok(())
        });

        let threads = GameEngineThreads {
            handle: handle_events.handle,
            handle_editor_input: handle_editor_input.handle,
            handle_game_input: handle_game_input.handle,
        };
        let senders = GameEngineSenders {
            editor_input: handle_editor_input.sender,
            game_input: handle_game_input.sender,
        };

        Ok(Self { threads, senders })
    }
}

pub struct GameEngineThreads {
    handle: ThreadHandle,
    handle_editor_input: ThreadHandle,
    handle_game_input: ThreadHandle,
}

impl GameEngineThreads {
    pub fn join(mut self) -> anyhow::Result<()> {
        self.handle.join()?;
        self.handle_editor_input.join()?;
        self.handle_game_input.join()?;

        Ok(())
    }
}

pub struct GameEngineSenders {
    pub editor_input: Sender<String>,
    pub game_input: Sender<GameInput>,
}

enum Event {
    EditorInput { line: String },
    GameInput(GameInput),
}

pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
