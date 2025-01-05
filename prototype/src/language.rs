use std::thread;

use tokio::{runtime::Runtime, select, sync::mpsc::error::SendError};

use crate::{
    channel::{self, Receiver},
    cli::Command,
    game_io::{GameInput, GameIo},
};

pub fn start(mut commands: Receiver<Command>) -> anyhow::Result<GameIo> {
    let runtime = Runtime::new()?;

    let (render_tx, mut render_rx) = channel::create();
    let (color_tx, color_rx) = channel::create();

    thread::spawn(move || {
        runtime.block_on(async {
            let mut code = Code {
                color: [0., 0., 0., 1.],
            };

            println!("Color: {:?}", code.color);

            loop {
                // The channel has no buffer, so this is synchronized to the
                // frame rate of the renderer.
                if let Err(SendError(_)) = color_tx.send(code.color) {
                    // The other end has hung up. Time for us to shut down too.
                    break;
                }

                let event = select! {
                    game_input = render_rx.recv() => {
                        let Some(game_input) = game_input else {
                            // The other end has hung up. We should shut down
                            // too.
                            break;
                        };

                        Event::GameInput(game_input)
                    }
                    command = commands.recv() => {
                        let Some(command) = command else {
                            // The other end has hung up. We should shut down
                            // too.
                            break;
                        };

                        Event::Command(command)
                    }
                };

                match event {
                    Event::Command(Command::SetColor { color }) => {
                        code.color = color;
                    }
                    Event::GameInput(GameInput::RenderingFrame) => {
                        // This loop is coupled to the frame rate of the
                        // renderer.
                    }
                }
            }
        });
    });

    Ok(GameIo {
        input: render_tx,
        output: color_rx,
    })
}

enum Event {
    Command(Command),
    GameInput(GameInput),
}

struct Code {
    color: [f64; 4],
}
