mod pass_fail;
mod test_report;
mod test_run_result;

use sycamore::{prelude::*, rt::JsCast};
use web_sys::{Event, KeyboardEvent};

use crate::{cp, render::test_run_result::TestRunResult};

pub fn render(test_reports: Vec<cp::TestReport>) {
    sycamore::render(|cx| {
        let input = create_signal(cx, String::new());
        let test_reports = create_signal(cx, test_reports);

        let detect_enter = |event: Event| {
            if let Some(event) = event.dyn_ref::<KeyboardEvent>() {
                if event.key() == "Enter" {
                    input.modify().clear();
                }
            }
        };

        view! { cx,
            input(
                bind:value=input,
                on:keyup=detect_enter,
                type="text",
                class="m-4 ring-1",
            )
            TestRunResult(test_reports=test_reports)
        }
    });
}
