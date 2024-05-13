use capi_runtime::{
    DebugEvent, Expression, ExpressionKind, InstructionAddress, Program,
    ProgramEffect, ProgramState,
};
use leptos::{
    component, create_memo, view, CollectView, IntoView, ReadSignal, SignalGet,
};
use web_sys::{wasm_bindgen::JsCast, HtmlSpanElement, MouseEvent};

use crate::{send_event, EventsTx};

#[component]
pub fn Function(
    program: ReadSignal<Option<Program>>,
    function: capi_runtime::Function,
    events: EventsTx,
) -> impl IntoView {
    let lines = function
        .syntax
        .into_iter()
        .map(|expression| {
            view! {
                <LineWithBreakpoint
                    program=program
                    expression=expression
                    events=events.clone() />
            }
        })
        .collect_view();

    view! {
        <div class="m-2 mb-4">
            <div class="font-bold">
                {function.name}:{'\n'}
            </div>
            <ol>
                {lines}
            </ol>
        </div>
    }
}

#[component]
pub fn LineWithBreakpoint(
    program: ReadSignal<Option<Program>>,
    expression: Expression,
    events: EventsTx,
) -> impl IntoView {
    let location = expression.location.clone();
    let address = create_memo(move |_| {
        program.get()?.source_map.location_to_address(&location)
    });

    let events2 = events.clone();
    let breakpoint = move || {
        let address = address.get()?;

        Some(view! {
            <Breakpoint
                program=program
                address=address
                events=events2.clone() />
        })
    };

    view! {
        <li class="ml-8">
            {breakpoint}
            <Expression
                program=program
                expression=expression
                events=events />
        </li>
    }
}

#[component]
pub fn Breakpoint(
    program: ReadSignal<Option<Program>>,
    address: InstructionAddress,
    events: EventsTx,
) -> impl IntoView {
    let class = move || {
        let program = program.get()?;

        let breakpoint_color = if program.breakpoint_at(&address) {
            "text-green-600"
        } else {
            "text-green-300"
        };

        Some(format!("mr-1 {breakpoint_color}"))
    };

    let data_address = move || address.to_usize();

    let toggle_breakpoint = move |event: MouseEvent| {
        let event_target = event.target().unwrap();
        let element = event_target.dyn_ref::<HtmlSpanElement>().unwrap();

        let address = element
            .get_attribute("data-address")
            .unwrap()
            .parse()
            .unwrap();

        leptos::spawn_local(send_event(
            DebugEvent::ToggleBreakpoint { address },
            events.clone(),
        ));
    };

    // It would be nicer to have the click handler on the enclosing element, to
    // make it less finicky for the user. But for some reason, I'm getting a
    // reference to the window on `event.current_target()`, so I have to rely on
    // `event.target()` to find the metadata. And that means, I can't have
    // events coming from multiple elements.
    //
    // There are probably better ways to solve this problem, but for now, this
    // is fine, if unfortunate.
    view! {
        <span
            class=class
            data-address=data_address
            on:click=toggle_breakpoint>
            {'⦿'}
        </span>
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

        let is_comment =
            matches!(expression.kind, ExpressionKind::Comment { .. });

        let text_classes = if is_comment {
            "italic text-gray-500"
        } else {
            ""
        };

        let bg_class = match &program.state {
            ProgramState::Effect { effect, address }
                if program.source_map.address_to_location(address).as_ref()
                    == Some(&expression.location) =>
            {
                match effect {
                    ProgramEffect::Paused => "bg-green-300",
                    _ => "bg-red-300",
                }
            }
            _ => {
                if let Some(address) = address {
                    if program.breakpoint_at(&address) {
                        "bg-blue-300"
                    } else {
                        ""
                    }
                } else {
                    ""
                }
            }
        };

        let class = Some(format!("px-0.5 {text_classes} {bg_class}"));

        let data_address = address.map(|address| address.to_usize());

        let toggle_breakpoint = move |event: MouseEvent| {
            let event_target = event.target().unwrap();
            let element = event_target.dyn_ref::<HtmlSpanElement>().unwrap();

            let address = element
                .get_attribute("data-address")
                .unwrap()
                .parse()
                .unwrap();

            leptos::spawn_local(send_event(
                DebugEvent::ToggleBreakpoint { address },
                events.clone(),
            ));
        };

        let line = format!("{}", expression.kind);

        Some(view! {
            <span
                class=class
                data-address=data_address
                on:click=toggle_breakpoint>
                {line}
            </span>
        })
    }
}
