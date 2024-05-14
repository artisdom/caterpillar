use capi_runtime::{
    DebugEvent, Expression, ExpressionKind, Program, ProgramEffect,
    ProgramState,
};
use leptos::{component, view, CollectView, IntoView, ReadSignal, SignalGet};
use web_sys::{wasm_bindgen::JsCast, HtmlSpanElement, MouseEvent};

use crate::client::{send_event, EventsTx};

#[component]
pub fn Function(
    program: ReadSignal<Option<Program>>,
    function: capi_runtime::Function,
    events: EventsTx,
) -> impl IntoView {
    let expressions = function
        .syntax
        .into_iter()
        .map(|expression| {
            view! {
                <li class="ml-8">
                    <Expression
                        program=program
                        expression=expression
                        events=events.clone() />
                </li>
            }
        })
        .collect_view();

    view! {
        <div class="m-2 mb-4">
            <div class="font-bold">
                {function.name}:{'\n'}
            </div>
            <ol>
                {expressions}
            </ol>
        </div>
    }
}

#[component]
pub fn Expression(
    program: ReadSignal<Option<Program>>,
    expression: Expression,
    events: EventsTx,
) -> impl IntoView {
    move || {
        // Without this line, this closure turns into an `FnOnce` instead of an
        // `Fn`, and that's no longer an a `leptos::IntoView`.
        let events = events.clone();

        let program = program.get()?;

        let address =
            program.source_map.location_to_address(&expression.location);

        let class_outer = if let Some(address) = address {
            if program.breakpoints.durable_breakpoint_at(&address) {
                "bg-blue-300"
            } else {
                ""
            }
        } else {
            ""
        };

        let class_inner = {
            let is_comment =
                matches!(expression.kind, ExpressionKind::Comment { .. });

            let text_classes = if is_comment {
                "italic text-gray-500"
            } else {
                ""
            };

            let bg_class = match &program.state {
                ProgramState::Effect { effect, address }
                    if program
                        .source_map
                        .address_to_location(address)
                        .as_ref()
                        == Some(&expression.location) =>
                {
                    match effect {
                        ProgramEffect::Paused => "bg-green-300",
                        _ => "bg-red-300",
                    }
                }
                _ => "",
            };

            Some(format!("px-0.5 {text_classes} {bg_class}"))
        };

        let data_address = address.map(|address| address.to_usize());

        let toggle_breakpoint = move |event: MouseEvent| {
            let event_target = event.target().unwrap();
            let element = event_target.dyn_ref::<HtmlSpanElement>().unwrap();

            let Some(address) = element.get_attribute("data-address") else {
                // This happens, if the user clicks on a comment.
                return;
            };
            let address = address.parse().unwrap();

            leptos::spawn_local(send_event(
                DebugEvent::ToggleBreakpoint { address },
                events.clone(),
            ));
        };

        let expression = format!("{}", expression.kind);

        Some(view! {
            <span class=class_outer>
                <span
                    class=class_inner
                    data-address=data_address
                    on:click=toggle_breakpoint>
                    {expression}
                </span>
            </span>
        })
    }
}
