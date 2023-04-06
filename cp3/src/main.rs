mod cp;
mod tests;

fn main() {
    let test_reports = tests::run();

    for test_report in test_reports {
        match &test_report.result {
            Ok(()) => {
                print!("PASS");
            }
            Err(_) => {
                print!("FAIL");
            }
        }

        print!(" {}", test_report.name);

        if let Err(err) = &test_report.result {
            print!("\n    {err}");
        }

        println!();
    }
}
