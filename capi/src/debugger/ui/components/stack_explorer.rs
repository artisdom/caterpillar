use leptos::{component, view, CollectView, IntoView};

use crate::{debugger::ui::components::panel::Panel, runtime::Operands};

#[allow(unused_braces)] // working around a warning from the `view!` macro
#[component]
pub fn StackExplorer(current: Operands) -> impl IntoView {
    view! {
        <Panel class="h-32">
            <div>
                <p>
                    "Current data stack:"
                </p>
                <Operands data_stack=current />
            </div>
        </Panel>
    }
}

#[component]
pub fn Operands(data_stack: Operands) -> impl IntoView {
    let values = data_stack
        .values()
        .map(|value| {
            view! {
                <li class="inline-block mr-2">{value.to_string()}</li>
            }
        })
        .collect_view();

    view! {
        <ol>
            {values}
        </ol>
    }
}
