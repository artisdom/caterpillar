use crate::model::{
    tests::infra::{
        debugger, ActiveFunctionsEntriesExt, ActiveFunctionsExt,
        DebugFunctionExt, FunctionsExt,
    },
    UserAction,
};

#[test]
fn display_breakpoint_that_was_set() -> anyhow::Result<()> {
    // Breakpoints that are set in the debugger state should be displayed.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: { |size_x size_y|
                    nop # this is where the breakpoint will be set
                    brk # prevent process from ending before we set breakpoint
                }
            ",
        )
        .run_program();

    let fragments = debugger.expect_code();
    let nop = fragments
        .find_function_by_name("main")
        .unwrap()
        .expect_one_branch()
        .iter(fragments)
        .next()
        .unwrap()
        .id();

    assert!(!debugger.expect_fragment(&nop).data.has_durable_breakpoint);

    debugger.on_user_action(UserAction::BreakpointSet { fragment: nop })?;
    assert!(debugger.expect_fragment(&nop).data.has_durable_breakpoint);

    Ok(())
}

#[test]
fn set_breakpoint_and_stop_there() -> anyhow::Result<()> {
    // If a breakpoint has been set, the program should run up until there, then
    // stop.

    let mut debugger = debugger();
    debugger.provide_source_code(
        r"
            main: { |size_x size_y|
                nop
            }
        ",
    );

    let fragments = debugger.expect_code();
    let nop = fragments
        .find_function_by_name("main")
        .unwrap()
        .expect_one_branch()
        .iter(fragments)
        .next()
        .unwrap()
        .id();
    debugger.on_user_action(UserAction::BreakpointSet { fragment: nop })?;

    debugger.run_program();

    assert_eq!(
        debugger
            .state
            .generate_transient_state()
            .active_functions
            .expect_entries()
            .expect_functions()
            .with_name("main")
            .active_fragment()
            .data
            .id,
        nop,
    );

    Ok(())
}

#[test]
#[should_panic] // https://github.com/hannobraun/caterpillar/issues/52
fn step_into_function() {
    // When stopping at a function call and then stepping, we expect to land at
    // the first fragment in the function.

    let mut debugger = debugger();
    debugger.provide_source_code(
        r"
            main: { |size_x size_y|
                1 2 3 f
            }

            # Add some arguments. In case the compiler decides to generate code
            # to handle those, this makes sure we step over that generated code.
            f: { |1 2 3|
                nop
            }
        ",
    );

    let fragments = debugger.expect_code();
    let f = fragments
        .find_function_by_name("main")
        .unwrap()
        .expect_one_branch()
        .iter(fragments)
        .nth(3)
        .unwrap()
        .id();
    debugger
        .on_user_action(UserAction::BreakpointSet { fragment: f })
        .unwrap();

    debugger.run_program();
    debugger.on_user_action(UserAction::Step).unwrap();

    assert_eq!(
        debugger
            .state
            .generate_transient_state()
            .active_functions
            .expect_entries()
            .expect_functions()
            .with_name("f")
            .active_fragment()
            .data
            .id,
        f,
    );
}
