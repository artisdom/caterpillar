use crate::{
    breakpoints::Breakpoints,
    runtime::{
        self, DataStack, Evaluator, EvaluatorEffect, EvaluatorState, Stack,
        Value,
    },
};

use super::{Event, State};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process {
    state: State,
    events: Vec<Event>,

    pub stack: Stack,
    pub evaluator: Evaluator,
    pub entry: runtime::Function,
    pub arguments: Vec<Value>,
}

impl Process {
    pub fn new(
        code: runtime::Code,
        entry: runtime::Function,
        arguments: Vec<Value>,
    ) -> Self {
        let mut self_ = Self {
            state: State::default(),
            events: Vec::new(),
            stack: Stack::new(entry.clone()),
            evaluator: Evaluator::new(code),
            entry,
            arguments: arguments.clone(),
        };

        self_.push(arguments);

        self_
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn stack(&self) -> &runtime::Stack {
        &self.stack
    }

    pub fn next_instruction(&self) -> Option<runtime::Location> {
        self.stack.next_instruction()
    }

    pub fn data_stack(&self) -> &DataStack {
        &self.stack.top_frame().unwrap().data
    }

    pub fn handle_first_effect(&mut self) {
        self.emit_event(Event::EffectHandled);
    }

    pub fn reset(&mut self) {
        self.evaluator.reset(self.entry.clone());
        self.state = State::default();

        self.push(self.arguments.clone());
    }

    pub fn push(&mut self, values: impl IntoIterator<Item = Value>) {
        for value in values {
            self.stack.top_frame_mut().unwrap().data.push(value);
        }
    }

    pub fn step(&mut self, breakpoints: &mut Breakpoints) {
        if !self.state.can_step() {
            return;
        }

        let next_instruction = self.next_instruction().unwrap();
        if breakpoints
            .should_stop_at_and_clear_ephemeral(next_instruction.clone())
        {
            self.emit_event(Event::EffectTriggered {
                effect: EvaluatorEffect::Builtin(
                    runtime::BuiltinEffect::Breakpoint,
                ),
            });
        }

        match self.evaluator.step(&mut self.stack) {
            Ok(EvaluatorState::Running) => self.emit_event(Event::HasStepped {
                location: next_instruction,
            }),
            Ok(EvaluatorState::Finished) => {
                self.emit_event(Event::Finished);
            }
            Err(effect) => {
                self.emit_event(Event::EffectTriggered { effect });
            }
        };
    }

    pub fn take_events(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.events.drain(..)
    }

    fn emit_event(&mut self, event: Event) {
        self.events.push(event.clone());
        self.state.evolve(event);
    }
}
