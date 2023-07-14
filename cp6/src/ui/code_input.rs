use sycamore::{
    component,
    flow::Indexed,
    reactive::{create_memo, create_signal, Scope, Signal},
    rt::JsCast,
    view,
    view::View,
    web::Html,
    Prop,
};
use web_sys::{Event, KeyboardEvent};

use crate::cp;

#[component]
pub fn CodeInput<'r, G: Html>(cx: Scope<'r>, mut props: Props<'r>) -> View<G> {
    let input = create_signal(cx, String::new());
    let error = create_signal(cx, String::new());

    let detect_enter = move |event: Event| {
        if let Some(event) = event.dyn_ref::<KeyboardEvent>() {
            if event.key() == "Enter" {
                let code = input.get();
                input.modify().clear();

                error.modify().clear();
                if let Err(err) = props.test_runner.run_code(&code) {
                    error.set(err.to_string());
                }
                let reports = props.test_runner.run_tests();

                props.test_reports.set(reports);
            }
        }
    };

    let error_lines = create_memo(cx, || {
        let error = error.get();
        error
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
    });

    view! { cx,
        div(class="flex flex-col") {
            input(
                bind:value=input,
                on:keyup=detect_enter,
                type="text",
                class="m-4 ring-1",
                autofocus=true,
            )
            div(class="max-w-fit max-h-fit") {
                Indexed(
                    iterable=error_lines,
                    view=|cx, line| view! { cx, p { (line) } },
                )
            }
        }
    }
}

#[derive(Prop)]
pub struct Props<'r> {
    test_runner: cp::TestRunner,
    test_reports: &'r Signal<cp::TestReports>,
}
