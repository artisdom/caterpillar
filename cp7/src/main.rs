mod args;
mod data_stack;
mod pipeline;

fn main() -> anyhow::Result<()> {
    let example = args::example()?;
    pipeline::run(example)?;
    Ok(())
}
