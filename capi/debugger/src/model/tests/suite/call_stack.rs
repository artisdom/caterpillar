use capi_runtime::Effect;

use crate::model::{
    tests::infra::{
        debugger, ActiveFunctionsEntriesExt, ActiveFunctionsExt,
        DebugBranchExt, DebugFragmentExt, DebugFunctionExt, FunctionsExt,
    },
    ActiveFunctionsEntry, UserAction,
};

#[test]
fn basic_call_stack() {
    // All functions in the call stack should show up in the active functions
    // view, if the process is stopped. This test constructs a scenario that
    // requires no special handling to detect and fix the effects of tail call
    // elimination, to provide a baseline.
    //
    // This test expects all defined functions to be active functions. The order
    // of functions is inner to outer, as it's most useful to the developer to
    // display the instruction where we're currently paused up top.

    let transient = debugger()
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    f
                    nop # make sure the previous call is not a tail call
                }
                f: { \ ->
                    g
                    nop # make sure the previous call is not a tail call
                }
                g: { \ ->
                    brk
                }
            ",
        )
        .run_program()
        .transient_state();

    let names = transient.active_functions.names();
    assert_eq!(names, vec!["g", "f", "main"]);
}

#[test]
fn stopped_at_host_function() {
    // If execution is stopped at a host function, it should be displayed as
    // such.

    let transient = debugger()
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    halt
                }
            ",
        )
        .run_program()
        .transient_state();

    transient
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("main")
        .active_fragment()
        .expect_call_to_host_function("halt");
}

#[test]
fn stopped_in_anonymous_function() {
    // If execution is stopped within an anonymous function, the function that
    // contains that block should appear as an active function, and the current
    // instruction should be visible.

    let transient = debugger()
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    { \ -> brk } eval
                }
            ",
        )
        .run_program()
        .transient_state();

    let fragment = transient
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("main")
        .only_branch()
        .fragment(0)
        .expect_function()
        .only_branch()
        .fragment(0);
    assert_eq!(fragment.data.effect, Some(Effect::Breakpoint));

    fragment.expect_call_to_intrinsic("brk");
}

#[test]
fn call_stack_reconstruction_missing_main() {
    // Tail call elimination can leave gaps in the call stack. If the `main`
    // function is missing due to that, it should be reconstructed.

    let transient = debugger()
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    f
                }

                f: { \ ->
                    brk
                }
            ",
        )
        .run_program()
        .transient_state();

    let names = transient.active_functions.names();
    assert_eq!(names, vec!["f", "main"]);

    transient
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("main")
        .active_fragment()
        .expect_call_to_function("f");
}

#[test]
fn call_stack_reconstruction_missing_single_branch_function() {
    // Tail call elimination can leave gaps in the call stack. If the missing
    // functions have only a single branch each, it is possible to add them back
    // without any additional hints being required.

    let transient = debugger()
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    f
                    nop # make sure the previous call is not a tail call
                }

                f: { \ ->
                    g
                }

                g: { \ ->
                    brk
                }
            ",
        )
        .run_program()
        .transient_state();

    let names = transient.active_functions.names();
    assert_eq!(names, vec!["g", "f", "main"]);

    transient
        .active_functions
        .expect_entries()
        .expect_functions()
        .with_name("f")
        .active_fragment()
        .expect_call_to_function("g");
}

#[test]
fn display_gap_where_missing_function_is_called_from_multi_branch_function() {
    // Tail call elimination can leave gaps in the call stack. Some simpler
    // cases are already getting reconstructed, but right now, we're not doing
    // that yet for functions with multiple branches.

    let transient = debugger()
        .provide_source_code(
            r"
                main: {
                    \ 0 0 ->
                        f

                    \ size_x size_y ->
                        f
                }

                f: { \ ->
                    g
                }

                g: { \ ->
                    brk
                }
            ",
        )
        .run_program()
        .transient_state();

    let entries = transient.active_functions.expect_entries();
    assert!(matches!(
        dbg!(entries.as_slice()),
        &[
            ActiveFunctionsEntry::Function(_),
            ActiveFunctionsEntry::Gap,
            ActiveFunctionsEntry::Function(_),
        ]
    ));
}

#[test]
#[should_panic] // https://github.com/hannobraun/caterpillar/issues/47
fn display_gap_where_missing_fn_is_called_from_reconstructed_multi_branch_fn() {
    // Tail call elimination can leave gaps in the call stack. Some simpler
    // cases are already getting reconstructed, but right now, we're not doing
    // that yet for anonymous functions with multiple branches.

    let transient = debugger()
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    0 f
                }

                f: {
                    \ 0 ->
                        g

                    \ n ->
                        g
                }

                g: { \ ->
                    h
                }

                h: { \ ->
                    brk
                }
            ",
        )
        .run_program()
        .transient_state();

    let entries = transient.active_functions.expect_entries();
    assert!(matches!(
        dbg!(entries.as_slice()),
        &[
            ActiveFunctionsEntry::Function(_),
            ActiveFunctionsEntry::Function(_),
            ActiveFunctionsEntry::Gap,
            ActiveFunctionsEntry::Function(_),
        ]
    ));
}

#[test]
fn instruction_on_call_stack_with_no_associated_fragment() {
    // If a host function has just been executed, it is possible that the
    // currently active instruction is a return instruction, if that is located
    // right after the call to the host function.
    //
    // Since a return is not associated with a fragment, the debugger must not
    // rely on that being the case. Everything should work normally in this
    // case.
    //
    // Note that as of this writing, there is an explicit test about host
    // functions on the stack. But that is stopped _at_ the host function, not
    // right after it. So it doesn't test the same thing.

    let mut debugger = debugger();
    debugger
        .provide_source_code(
            r"
                main: { \ size_x size_y ->
                    submit
                    main
                }

                submit: { \ ->
                    submit_frame # this is the call to the host function
                }
            ",
        )
        .run_program();

    // This will stop the program after the `submit_frame`, since that is the
    // only time when the game engine reacts to commands.
    debugger.on_user_action(UserAction::Stop).unwrap();

    let _ = debugger.transient_state();

    // Nothing else to do. We're not currently handling this case well[1], so
    // unless we panicked, everything's good, as far as this test is concerned.
    //
    // [1] https://github.com/hannobraun/caterpillar/issues/53
}
