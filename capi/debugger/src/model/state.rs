use capi_compiler::fragments::{FragmentId, FragmentLocation};
use capi_game_engine::{command::Command, memory::Memory};
use capi_protocol::{
    host_state::HostState,
    updates::{Code, UpdateFromHost},
};
use capi_runtime::{Effect, Instruction, Instructions, Value};

use super::{
    ActiveFunctions, Breakpoints, DebugCode, DebugFragmentKind, UserAction,
};

#[derive(Clone, Debug, Default)]
pub struct PersistentState {
    pub code: DebugCode,
    pub breakpoints: Breakpoints,
    pub host_state: Option<HostState>,
    pub memory: Option<Memory>,
}

impl PersistentState {
    pub fn on_new_code(&mut self, code: Code) -> Command {
        let instructions = self.apply_breakpoints(&code);
        self.code.inner = Some(code);
        Command::UpdateCode { instructions }
    }

    pub fn on_update_from_host(&mut self, update: UpdateFromHost) {
        match update {
            UpdateFromHost::Memory { memory } => {
                self.memory = Some(memory);
            }
            UpdateFromHost::State { state } => {
                self.host_state = Some(state);
            }
        }
    }

    pub fn on_user_action(
        &mut self,
        action: UserAction,
        transient: &TransientState,
    ) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        match action {
            UserAction::BreakpointClear {
                fragment: (fragment, _),
                ..
            } => {
                let code = self.code.get()?;
                let address = self.code.fragment_to_instruction(&fragment)?;

                self.breakpoints.clear_durable(&address);

                commands.push(Command::UpdateCode {
                    instructions: self.apply_breakpoints(code),
                });
            }
            UserAction::BreakpointSet {
                fragment: (fragment, _),
                ..
            } => {
                let code = self.code.get()?;
                let address = self.code.fragment_to_instruction(&fragment)?;

                self.breakpoints.set_durable(address);

                commands.push(Command::UpdateCode {
                    instructions: self.apply_breakpoints(code),
                });
            }
            UserAction::Continue => {
                let origin = &transient
                    .active_functions
                    .entries()?
                    .leaf()
                    .function()?
                    .active_branch()?
                    .active_fragment()?
                    .data;
                let targets = Vec::new();

                self.step_or_continue(
                    (&origin.id, &origin.location),
                    targets,
                    &mut commands,
                )?;
            }
            UserAction::Reset => {
                commands.push(Command::Reset);
            }
            UserAction::StepIn => {
                let code = self.code.get()?;

                let entries = transient.active_functions.entries()?;
                let branch = entries.leaf().function()?.active_branch()?;

                let origin = branch.active_fragment()?;
                let targets = if let Some(function) =
                    origin.data.fragment.as_call_to_function(&code.fragments)
                {
                    function
                        .branches
                        .values()
                        .filter_map(|branch| branch.start)
                        .collect()
                } else {
                    let mut fragment = origin.clone();

                    loop {
                        let Some(after) = entries
                            .find_next_fragment_or_next_after_caller(
                                branch,
                                &fragment.data.id,
                            )?
                        else {
                            // Can't find a next fragment _or_ a caller, which
                            // means we must be at the top-level function.
                            //
                            // Let's just tell the runtime to continue, so the
                            // process finishes.
                            self.step_or_continue(
                                (&origin.data.id, &origin.data.location),
                                vec![],
                                &mut commands,
                            )?;
                            return Ok(commands);
                        };

                        if let DebugFragmentKind::Comment { .. } = after.kind {
                            // Can't step to comments! Need to ignore them.
                            fragment = after.clone();
                            continue;
                        }

                        break vec![after.data.id];
                    }
                };

                self.step_or_continue(
                    (&origin.data.id, &origin.data.location),
                    targets,
                    &mut commands,
                )?;
            }
            UserAction::StepOut => {
                let entries = transient.active_functions.entries()?;
                let origin = entries
                    .leaf()
                    .function()?
                    .active_branch()?
                    .active_fragment()?;

                let targets = {
                    let mut fragment = origin.clone();

                    loop {
                        let Some(after) = entries
                            .find_next_fragment_after_caller(
                                &fragment.data.id,
                            )?
                        else {
                            // Can't find a next fragment _or_ a caller, which
                            // means we must be at the top-level function.
                            //
                            // Let's just tell the runtime to continue, so the
                            // process finishes.
                            self.step_or_continue(
                                (&origin.data.id, &origin.data.location),
                                vec![],
                                &mut commands,
                            )?;
                            return Ok(commands);
                        };

                        if let DebugFragmentKind::Comment { .. } = after.kind {
                            // Can't step to comments! Need to ignore them.
                            fragment = after.clone();
                            continue;
                        }

                        break vec![after.data.id];
                    }
                };

                self.step_or_continue(
                    (&origin.data.id, &origin.data.location),
                    targets,
                    &mut commands,
                )?;
            }
            UserAction::StepOver => {
                let entries = transient.active_functions.entries()?;
                let branch = entries.leaf().function()?.active_branch()?;

                let origin = branch.active_fragment()?;

                let targets = {
                    let mut fragment = origin.clone();

                    loop {
                        let Some(after) = entries
                            .find_next_fragment_or_next_after_caller(
                                branch,
                                &fragment.data.id,
                            )?
                        else {
                            // Can't find a next fragment _or_ a caller, which
                            // means we must be at the top-level function.
                            //
                            // Let's just tell the runtime to continue, so the
                            // process finishes.
                            self.step_or_continue(
                                (&origin.data.id, &origin.data.location),
                                vec![],
                                &mut commands,
                            )?;
                            return Ok(commands);
                        };

                        if let DebugFragmentKind::Comment { .. } = after.kind {
                            // Can't step to comments! Need to ignore them.
                            fragment = after.clone();
                            continue;
                        }

                        break vec![after.data.id];
                    }
                };

                self.step_or_continue(
                    (&origin.data.id, &origin.data.location),
                    targets,
                    &mut commands,
                )?;
            }
            UserAction::Stop => {
                commands.push(Command::Stop);
            }
        };

        Ok(commands)
    }

    pub fn generate_transient_state(&self) -> TransientState {
        let active_functions = ActiveFunctions::new(
            self.code.inner.as_ref(),
            &self.breakpoints,
            self.host_state.as_ref(),
        );
        let operands = match &self.host_state {
            Some(HostState::Stopped {
                current_operands, ..
            }) => current_operands.clone(),
            _ => Vec::new(),
        };

        TransientState {
            active_functions,
            operands,
        }
    }

    fn step_or_continue(
        &mut self,
        (origin, _): (&FragmentId, &FragmentLocation),
        targets: Vec<FragmentId>,
        commands: &mut Vec<Command>,
    ) -> anyhow::Result<()> {
        let origin = self.code.fragment_to_instruction(origin)?;
        let code = self.code.get()?;

        // Whatever happens next, any ephemeral breakpoints that were used to
        // get us here are obsolete. Let's get rid of them.
        //
        // Since stepping or continuing happens when the runtime is stopped,
        // this might remove an ephemeral breakpoint at the current instruction.
        // (It might not, if we're stopped because of a `brk` or a durable
        // breakpoint.)
        //
        // This might also remove ephemeral breakpoints in other places, like
        // sibling branches. When stepping into branches, the debugger doesn't
        // know where we'll end up, so it sets ephemeral breakpoints in all of
        // the branches.
        self.breakpoints.clear_all_ephemeral();

        // And of course, if we have any targets we want to stop at (we might
        // not, if we're continuing instead of stepping), we need to set
        // ephemeral breakpoints there.
        for target in targets {
            let target = self.code.fragment_to_instruction(&target)?;
            self.breakpoints.set_ephemeral(target);
        }

        // We might have a durable breakpoint at the instruction we're trying to
        // step over. We need to remove that before we can proceed.
        let durable_breakpoint_at_origin =
            self.breakpoints.clear_durable(&origin);

        // We're done setting and clearing breakpoints, for now. Let's apply
        // them to the current code, to get instructions we can send to the
        // runtime.
        let mut instructions = self.apply_breakpoints(code);

        // If the instruction we are about to step over is a `brk`, that won't
        // ever do anything except trigger another breakpoint.
        //
        // In that case, we need to replace the instruction with a `nop` before
        // attempting to step over it.
        if let Instruction::TriggerEffect {
            effect: Effect::Breakpoint,
        } = self.code.instruction(&origin)?
        {
            // We don't need to explicitly revert this with another replacement
            // later, as we'll re-apply breakpoints based on the original code.
            instructions.replace(&origin, Instruction::Nop);
        }

        // Apply all changes to the code and move on to the next instruction.
        // The latter might be redundant with the continue below. But that
        // should be just that, redundant. It shouldn't lead to a different
        // result.
        commands.extend([
            Command::UpdateCode { instructions },
            Command::ClearBreakpointAndEvaluateNextInstruction,
        ]);

        // In case we removed a durable breakpoint, we need to revert that.
        if durable_breakpoint_at_origin {
            self.breakpoints.set_durable(origin);
        }

        // And finally, we can provide the latest code to the runtime, then send
        // it on its way.
        commands.extend([
            Command::UpdateCode {
                instructions: self.apply_breakpoints(code),
            },
            Command::ClearBreakpointAndContinue,
        ]);

        Ok(())
    }

    fn apply_breakpoints(&self, code: &Code) -> Instructions {
        let mut instructions = code.instructions.clone();

        for address in self.breakpoints.iter() {
            instructions.replace(
                &address,
                Instruction::TriggerEffect {
                    effect: Effect::Breakpoint,
                },
            );
        }

        instructions
    }
}

#[derive(Clone, Debug)]
pub struct TransientState {
    pub active_functions: ActiveFunctions,
    pub operands: Vec<Value>,
}
