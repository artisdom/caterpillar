mod cp;
mod render;

fn main() -> anyhow::Result<()> {
    let (mut functions, tests) = cp::define_code()?;
    let test_reports = cp::run_tests(&mut functions, &tests)?;
    render::render(test_reports);

    Ok(())
}
