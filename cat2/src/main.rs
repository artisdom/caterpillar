mod cells;
mod cp;
mod event_loop;
mod ui;

use std::{io, time::Duration};

use crossterm::{
    event::{
        Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
    },
    terminal,
};
use futures::StreamExt;
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut events = EventStream::new();

    let mut state = event_loop::State {
        functions: cp::Functions::new(),
        generations: Vec::new(),
        buffer: ui::Buffer::new(),
        stdout: io::stdout(),
    };

    let delay = Duration::from_millis(125);
    let mut interval = time::interval(delay);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = interval.tick() => {}
            event = events.next() => {
                let Some(event) = event else {
                    anyhow::bail!("Error reading input event");
                };
                let event = event?;

                if let Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                }) = event
                {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        // CTRL-C
                        break;
                    }
                }
            }
        }

        let current = state
            .generations
            .last()
            .cloned()
            .unwrap_or_else(cells::init);

        // We only add new generations, but never delete them. This is fine for
        // now, I think. Let's just hope nobody runs this for long enough to
        // fill up their main memory.
        let next = cells::next_generation(current, &state.functions);
        state.generations.push(next);

        ui::draw(
            &state.generations,
            &state.functions,
            &mut state.buffer,
            &mut state.stdout,
        )?;
    }

    terminal::disable_raw_mode()?;

    Ok(())
}
