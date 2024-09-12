use std::mem;

use crate::{
    breakpoints::Breakpoints, evaluator::Evaluator,
    instructions::InstructionAddress, Effect, Effects, Instructions, Stack,
    Value,
};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct Process {
    most_recent_step: Option<InstructionAddress>,
    effects: Effects,
    evaluator: Evaluator,
    breakpoints: Breakpoints,
}

impl Process {
    pub fn most_recent_step(&self) -> Option<InstructionAddress> {
        self.most_recent_step
    }

    pub fn state(&self) -> ProcessState {
        if self.effects().inspect_first().is_some() {
            ProcessState::Stopped
        } else if self.evaluator.stack.no_frames_left() {
            ProcessState::Finished
        } else {
            ProcessState::Running
        }
    }

    pub fn effects(&self) -> &Effects {
        &self.effects
    }

    pub fn effects_mut(&mut self) -> &mut Effects {
        &mut self.effects
    }

    pub fn evaluator(&self) -> &Evaluator {
        &self.evaluator
    }

    pub fn stack(&self) -> &Stack {
        &self.evaluator.stack
    }

    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.evaluator.stack
    }

    pub fn breakpoints(&self) -> &Breakpoints {
        &self.breakpoints
    }

    pub fn breakpoints_mut(&mut self) -> &mut Breakpoints {
        &mut self.breakpoints
    }

    pub fn reset(&mut self, arguments: impl IntoIterator<Item = Value>) {
        // All we need to preserve when we reset are the breakpoints. Anything
        // else needs to go back to start conditions.
        //
        // Doing it like this, as opposed to just resetting all other fields,
        // has the advantage that this code doesn't need to be changed in sync
        // with new fields being added.
        let breakpoints = mem::take(&mut self.breakpoints);
        *self = Self {
            breakpoints,
            ..Self::default()
        };

        for argument in arguments {
            self.evaluator.stack.push_operand(argument);
        }
    }

    pub fn continue_(&mut self, and_stop_at: Option<InstructionAddress>) {
        if let Some(Effect::Breakpoint) = self.effects().inspect_first() {
            if let Some(address) = and_stop_at {
                self.breakpoints.set_ephemeral(address);
            }

            self.effects_mut().handle_first();
        }
    }

    pub fn stop(&mut self) {
        self.effects_mut().trigger(Effect::Breakpoint);
    }

    pub fn step(&mut self, instructions: &Instructions) {
        if !self.state().is_running() {
            return;
        }

        let next_instruction = self.evaluator.next_instruction;

        if self
            .breakpoints
            .should_stop_at_and_clear_ephemeral(&next_instruction)
        {
            self.effects_mut().trigger(Effect::Breakpoint);
            return;
        }

        if let Err(effect) = self.evaluator.step(instructions) {
            self.effects_mut().trigger(effect);
        }

        self.most_recent_step = Some(next_instruction);
    }
}

pub enum ProcessState {
    Running,
    Finished,
    Stopped,
}

impl ProcessState {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn has_finished(&self) -> bool {
        matches!(self, Self::Finished)
    }
}
