use std::{io::stdin, thread};

use tokio::{
    runtime::Runtime,
    select,
    sync::mpsc::{self, error::SendError},
};

use crate::game_io::{GameInput, GameIo};

pub fn start_in_background() -> anyhow::Result<GameIo> {
    let runtime = Runtime::new()?;

    let (render_tx, mut render_rx) = mpsc::unbounded_channel();
    let (color_tx, color_rx) = mpsc::unbounded_channel();
    let (commands_tx, mut commands_rx) = mpsc::unbounded_channel();

    thread::spawn(move || {
        runtime.block_on(async {
            let color = [0., 0., 0., 1.];

            println!("Color: {color:?}");

            loop {
                // The channel has no buffer, so this is synchronized to the
                // frame rate of the renderer.
                if let Err(SendError(_)) = color_tx.send(color) {
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
                    command = commands_rx.recv() => {
                        let _ = command;
                        continue;
                    }
                };

                match event {
                    Event::GameInput(GameInput::RenderingFrame) => {
                        // This loop is coupled to the frame rate of the
                        // renderer.
                    }
                }
            }
        });
    });

    // We're using Tokio here and could use its asynchronous stdio API. But the
    // Tokio documentation explicitly recommends against using that for
    // interactive code, recommending a dedicated thread instead.
    thread::spawn(move || loop {
        let mut command = String::new();
        stdin().read_line(&mut command).unwrap();
        if let Err(SendError(_)) = commands_tx.send(command) {
            // The other end has hung up. We should shut down too.
            break;
        }
    });

    Ok(GameIo {
        input: render_tx,
        output: color_rx,
    })
}

enum Event {
    GameInput(GameInput),
}
