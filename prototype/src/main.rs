mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (color_tx, color_rx) = actor::channel();

    let (input, commands) = language::start(color_tx)?;
    let mut cli = cli::start(commands.sender);
    game_io::start_and_wait(input.sender, color_rx)?;

    cli.join()?;

    Ok(())
}
