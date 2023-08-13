use crate::language::{
    intrinsics,
    pipeline::{self, PipelineError},
    syntax::Syntax,
    tokens::Tokens,
};

use super::{
    evaluator::{Evaluator, EvaluatorError, EvaluatorState},
    functions::Intrinsic,
    updater,
};

#[derive(Debug)]
pub struct Interpreter {
    pub syntax: Syntax,
    pub evaluator: Evaluator,
    pub tokens: Tokens,
}

impl Interpreter {
    pub fn new(code: &str) -> Result<Self, PipelineError> {
        let mut syntax = Syntax::new();
        let (start, tokens) = pipeline::run(code, &mut syntax)?;

        let mut evaluator = Evaluator::new();
        if let Some(start) = start {
            evaluator.call_stack.push(start);
        }

        let intrinsics = [
            ("+", intrinsics::add as Intrinsic),
            ("clone", intrinsics::clone),
            ("delay_ms", intrinsics::delay_ms),
            ("print_line", intrinsics::print_line),
            ("fn", intrinsics::fn_),
        ];

        for (name, intrinsic) in intrinsics {
            evaluator.functions.register_intrinsic(name, intrinsic)
        }

        Ok(Interpreter {
            syntax,
            evaluator,
            tokens,
        })
    }

    pub fn step(&mut self) -> Result<EvaluatorState, EvaluatorError> {
        self.evaluator.step(&self.syntax)
    }

    pub fn update(&mut self, code: &str) -> Result<(), PipelineError> {
        self.syntax.prepare_update();
        let (_, tokens) = pipeline::run(code, &mut self.syntax)?;
        updater::update(&self.syntax, &mut self.evaluator);
        self.tokens = tokens;

        Ok(())
    }
}
