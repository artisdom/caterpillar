use crate::{
    breakpoints::Breakpoints,
    runtime::{
        self, DataStack, Evaluator, EvaluatorEffect, EvaluatorState, Value,
    },
};

use super::{Event, State};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process {
    state: State,

    pub evaluator: Evaluator,
    pub entry: runtime::Function,
    pub arguments: Vec<Value>,

    /// The data stack, before the most recent instruction was executed
    pub previous_data_stack: DataStack,
}

impl Process {
    pub fn new(
        code: runtime::Code,
        entry: runtime::Function,
        arguments: Vec<Value>,
    ) -> Self {
        let mut evaluator = Evaluator::new(code, entry.clone());
        evaluator.push(arguments.clone());

        Self {
            state: State::default(),
            evaluator,
            entry,
            arguments,
            previous_data_stack: DataStack::default(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn handle_first_effect(&mut self) {
        self.emit_event(Event::HandleEffect);
    }

    pub fn reset(&mut self) {
        self.evaluator.reset(self.entry.clone());
        self.state = State::default();
        self.previous_data_stack.clear();

        self.push(self.arguments.clone());
    }

    pub fn push(&mut self, values: impl IntoIterator<Item = Value>) {
        self.evaluator.push(values);
    }

    pub fn step(&mut self, breakpoints: &mut Breakpoints) {
        if !self.state.can_step() {
            return;
        }

        let next_instruction = self.evaluator.next_instruction().unwrap();
        if breakpoints
            .should_stop_at_and_clear_ephemeral(next_instruction.clone())
        {
            self.emit_event(Event::TriggerEffect {
                effect: EvaluatorEffect::Builtin(
                    runtime::BuiltinEffect::Breakpoint,
                ),
            });
        }

        self.previous_data_stack =
            self.evaluator.stack().top_frame().unwrap().data.clone();
        match self.evaluator.step() {
            Ok(EvaluatorState::Running) => self.emit_event(Event::Step {
                location: next_instruction,
            }),
            Ok(EvaluatorState::Finished) => {
                self.emit_event(Event::Finish);
            }
            Err(effect) => {
                self.emit_event(Event::TriggerEffect { effect });
            }
        };
    }

    fn emit_event(&mut self, event: Event) {
        self.state.evolve(event);
    }
}
