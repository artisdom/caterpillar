mod active_functions;
mod debugger;
mod expression;
mod function;
mod remote_process;

pub use self::{
    active_functions::ActiveFunctions,
    debugger::Debugger,
    expression::{Expression, OtherExpression},
    function::Function,
    remote_process::RemoteProcess,
};

#[cfg(test)]
mod tests {
    use capi_compiler::{
        compile,
        repr::{fragments::FragmentExpression, syntax::Script},
    };
    use capi_process::{Bytecode, CoreEffect, Effect, Process};
    use capi_protocol::{
        host::GameEngineHost,
        memory::Memory,
        updates::{Code, Updates},
    };

    use crate::debugger::{
        active_functions::ActiveFunctionsMessage, ActiveFunctions, Expression,
        RemoteProcess,
    };

    use super::{Debugger, Function, OtherExpression};

    #[test]
    fn no_server() {
        // If `RemoteProcess` has received no updates at all, the active
        // functions view should display that no server is available.

        let debugger = init().to_debugger();

        assert_eq!(
            debugger.active_functions,
            ActiveFunctions::Message {
                message: ActiveFunctionsMessage::NoServer
            }
        );
        assert!(debugger.operands.is_none());
        assert!(debugger.memory.is_none());
    }

    #[test]
    fn no_process() {
        // If `RemoteProcess` has received a code update but no runtime updates,
        // the active functions view should display that no process is
        // available.

        let debugger = init().provide_source_code(|_| {}).to_debugger();

        assert_eq!(
            debugger.active_functions,
            ActiveFunctions::Message {
                message: ActiveFunctionsMessage::NoProcess
            }
        );
        assert!(debugger.operands.is_none());
        assert!(debugger.memory.is_none());
    }

    #[test]
    fn basic_call_stack() {
        // All functions in the call stack should show up in the active
        // functions view, if the process is stopped. This test constructs a
        // scenario that requires no special handling to detect and fix tail
        // call optimization, to provide a baseline.
        //
        // This test expects all defined functions to be active functions. The
        // order of functions is inner to outer, as it's most useful to the
        // developer to display the instruction where we're currently paused up
        // top.

        let debugger = init()
            .provide_source_code(|script| {
                script
                    .function("main", [], |s| {
                        s.r("f")
                            // Not triggered. Just here to prevent tail call
                            // optimization from removing this function from the
                            // call stack.
                            .r("brk");
                    })
                    .function("f", [], |s| {
                        s.r("g")
                            // Not triggered. Just here to prevent tail call
                            // optimization from removing this function from the
                            // call stack.
                            .r("brk");
                    })
                    .function("g", [], |s| {
                        s.r("brk");
                    });
            })
            .run_process()
            .to_debugger();

        let names = debugger
            .active_functions
            .expect_functions()
            .into_iter()
            .map(|active_function| active_function.name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["g", "f", "main"]);
    }

    #[test]
    fn stopped_at_code_within_block() {
        // If execution is stopped within a block, the function that contains
        // that block should appear as an active function, and the current
        // instruction should be visible.

        let debugger = init()
            .provide_source_code(|script| {
                script.function("main", [], |s| {
                    s.block(|s| {
                        s.r("brk");
                    })
                    .r("eval");
                });
            })
            .run_process()
            .to_debugger();

        let other = debugger
            .active_functions
            .expect_functions()
            .remove(0)
            .body
            .remove(0)
            .expect_block()
            .remove(0)
            .expect_other();
        assert_eq!(other.effect, Some(Effect::Core(CoreEffect::Breakpoint)));

        let builtin = other.expression.expect_builtin_function();
        assert_eq!(builtin, "brk");
    }

    #[test]
    fn active_function_has_been_tail_call_optimized() {
        // When a function calls another function, and that call is the last
        // expression in the calling function, the stack frame for the calling
        // function is removed from the call stack. This is called tail call
        // optimization, and it enables unlimited recursion.
        //
        // Optimizing away those stack frames has no effect on the running
        // process (except for limiting the memory use), because the stack frame
        // would have been removed anyway, after the called function returns.
        // However, if execution stops, and the removed stack frames lead to
        // gaps in the "active functions" view in the debugger, this is
        // confusing to the developer, who no longer gets the full picture of
        // what's happening.
        //
        // Fortunately, it's possible recognize these gaps, and since the
        // debugger has access to the source code, it can figure out what was
        // missing and fill that in.

        let debugger = init()
            .provide_source_code(|script| {
                script
                    .function("main", [], |s| {
                        s.r("f")
                            // This is never triggered. It's just here, so the
                            // function call is not the last expression, because
                            // I don't want this function to be optimized away
                            // too.
                            .r("brk");
                    })
                    .function("f", [], |s| {
                        s.r("g");
                    })
                    .function("g", [], |s| {
                        s.r("brk");
                    });
            })
            .run_process()
            .to_debugger();

        let mut function =
            debugger.active_functions.expect_functions().remove(1);
        assert_eq!(function.name, "f");

        let call_to_g = function
            .body
            .remove(0)
            .expect_other()
            .expression
            .expect_user_function();
        assert_eq!(call_to_g, "g");
    }

    #[test]
    fn main_function_has_been_tail_call_optimized() {
        // This test is similar to the previous test, in that it concerns tail
        // call optimization, and how that prevents functions from showing up in
        // "active functions", even if they should be there.
        //
        // In this case, the function that was optimized away is the `main`
        // function. This isn't really any different from the perspective of the
        // compiler and process, but the debugger needs to detect this condition
        // in a different way.

        let debugger = init()
            .provide_source_code(|script| {
                script
                    .function("main", [], |s| {
                        s.r("f");
                    })
                    .function("f", [], |s| {
                        s.r("brk");
                    });
            })
            .run_process()
            .to_debugger();

        let mut function =
            debugger.active_functions.expect_functions().remove(1);
        assert_eq!(function.name, "main");

        let call_to_f = function
            .body
            .remove(0)
            .expect_other()
            .expression
            .expect_user_function();
        assert_eq!(call_to_f, "f");
    }

    fn init() -> TestSetup {
        TestSetup::default()
    }

    #[derive(Default)]
    struct TestSetup {
        remote_process: RemoteProcess,
        bytecode: Option<Bytecode>,
    }

    impl TestSetup {
        fn provide_source_code(
            &mut self,
            f: impl FnOnce(&mut Script),
        ) -> &mut Self {
            let mut script = Script::default();
            f(&mut script);

            let (fragments, bytecode, source_map) =
                compile::<GameEngineHost>(script);

            self.remote_process.on_code_update(Code {
                fragments: fragments.clone(),
                bytecode: bytecode.clone(),
                source_map,
            });

            self.bytecode = Some(bytecode);

            self
        }

        fn run_process(&mut self) -> &mut Self {
            let bytecode = self.bytecode.as_ref().expect(
                "Must provide source code via `TestSetup::source_code` before \
                running process.",
            );

            let mut process = Process::default();
            process.reset([]);
            while process.state().can_step() {
                process.step(bytecode);
            }

            let memory = Memory::default();
            let mut updates = Updates::default();

            updates.queue_updates(&process, &memory);
            for update in updates.take_queued_updates() {
                self.remote_process.on_runtime_update(update);
            }

            self
        }

        fn to_debugger(&self) -> Debugger {
            self.remote_process.to_debugger()
        }
    }

    trait ActiveFunctionsExt {
        fn expect_functions(self) -> Vec<Function>;
    }

    impl ActiveFunctionsExt for ActiveFunctions {
        fn expect_functions(self) -> Vec<Function> {
            let ActiveFunctions::Functions { functions } = self else {
                panic!("Expected active functions to be displayed");
            };

            functions
        }
    }

    trait ExpressionExt {
        fn expect_block(self) -> Vec<Expression>;
        fn expect_other(self) -> OtherExpression;
    }

    impl ExpressionExt for Expression {
        fn expect_block(self) -> Vec<Expression> {
            let Expression::Block { expressions } = self else {
                panic!("Expected block");
            };

            expressions
        }

        fn expect_other(self) -> OtherExpression {
            let Expression::Other(other) = self else {
                panic!("Expected other expression");
            };

            other
        }
    }

    trait FragmentExpressionExt {
        fn expect_builtin_function(self) -> String;
        fn expect_user_function(self) -> String;
    }

    impl FragmentExpressionExt for FragmentExpression {
        fn expect_builtin_function(self) -> String {
            let FragmentExpression::ResolvedBuiltinFunction { name } = self
            else {
                panic!("Expected builtin function");
            };

            name
        }

        fn expect_user_function(self) -> String {
            let FragmentExpression::ResolvedUserFunction { name } = self else {
                panic!("Expected user function");
            };

            name
        }
    }
}
