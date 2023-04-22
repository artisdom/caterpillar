mod cp;
mod functions;
mod tests;

use crossterm::style::Stylize;

fn main() -> anyhow::Result<()> {
    let mut functions = functions::define()?;
    let test_reports = tests::run(&mut functions)?;

    for test_report in test_reports {
        match &test_report.result {
            Ok(()) => {
                print!("{}", "PASS".bold().green());
            }
            Err(_) => {
                print!("{}", "FAIL".bold().red());
            }
        }

        print!(" {} - {}", test_report.module, test_report.name);

        if let Err(err) = &test_report.result {
            print!("\n    {err}");
        }

        println!();
    }

    Ok(())
}
