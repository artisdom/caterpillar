use std::collections::BTreeMap;

use crate::{
    operands::PopOperandError, Function, Instruction, InstructionAddress,
    Instructions, Operands, Value,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Stack {
    frames: Vec<StackElement>,

    /// # Special heap for closures
    ///
    /// ## Implementation Note
    ///
    /// This doesn't belong here. It just was a convenient place to put it, as
    /// all code that needs to deal with closures has access to `Stack`.
    ///
    /// The eventual plan is to put closures on the regular stack, but that is
    /// likely to be impractical while the language is untyped.
    pub closures: BTreeMap<u32, (InstructionAddress, BTreeMap<String, Value>)>,
    pub next_closure: u32,
}

impl Stack {
    pub fn new() -> Self {
        let frames =
            vec![StackElement::Frame(StackFrame::new(InstructionAddress {
                index: 0,
            }))];

        Self {
            frames,
            closures: BTreeMap::new(),
            next_closure: 0,
        }
    }

    pub fn bindings(&self) -> Option<&Bindings> {
        self.frames.last().map(|frame| match frame {
            StackElement::Frame(frame) => &frame.bindings,
        })
    }

    pub fn operands(&self) -> Option<&Operands> {
        self.frames.last().map(|frame| match frame {
            StackElement::Frame(frame) => &frame.operands,
        })
    }

    pub fn next_instruction_in_current_frame(
        &self,
    ) -> Option<InstructionAddress> {
        let StackElement::Frame(frame) = self.frames.last()?;
        Some(frame.next_instruction)
    }

    pub fn next_instruction_overall(&self) -> Option<InstructionAddress> {
        let StackElement::Frame(frame) = self.frames.last()?;
        Some(frame.next_instruction)
    }

    pub fn is_next_instruction_in_any_frame(
        &self,
        instruction: &InstructionAddress,
    ) -> bool {
        let mut instruction = *instruction;
        instruction.increment();

        self.frames.iter().any(|frame| {
            let StackElement::Frame(frame) = frame;
            frame.next_instruction == instruction
        })
    }

    pub fn all_next_instructions_in_frames(
        &self,
    ) -> impl DoubleEndedIterator<Item = InstructionAddress> + '_ {
        self.frames.iter().map(|frame| {
            let StackElement::Frame(frame) = frame;
            frame.next_instruction
        })
    }

    pub fn push_frame(
        &mut self,
        function: Function,
        instructions: &Instructions,
    ) -> Result<(), PushStackFrameError> {
        // We must create the new stack frame before we do tail call
        // optimization. Otherwise, we might drop the current frame, and if the
        // current frame is the top-level frame, then any potential arguments
        // for the new frame have nowhere to go.
        let mut new_frame = StackFrame::new(function.start);

        // Move arguments into the new frame.
        if let Some(StackElement::Frame(caller)) = self.frames.last_mut() {
            for argument in function.arguments.iter().rev() {
                let value = caller.operands.pop_any()?;
                new_frame.bindings.insert(argument.clone(), value);
            }
        } else {
            assert_eq!(
                function.arguments.len(),
                0,
                "Function has no caller, which means there is no stack frame \
                that the function could take its arguments from. Yet, it has \
                arguments, which can't work.",
            );
        }

        if let Some(next_addr) = self.next_instruction_in_current_frame() {
            let next_instruction = instructions
                .get(&next_addr)
                .expect("Expected instruction referenced on stack to exist");

            // If the current function is finished, pop its stack frame before
            // pushing the next one. This is tail call optimization.
            if let Instruction::Return = next_instruction {
                self.pop_frame()
                    .expect("Currently executing; stack can't be empty");
            }
        }

        const RECURSION_LIMIT: usize = 16;
        if self.frames.len() >= RECURSION_LIMIT {
            return Err(PushStackFrameError::Overflow);
        }

        self.frames.push(StackElement::Frame(new_frame));

        Ok(())
    }

    pub fn pop_frame(&mut self) -> Result<(), StackIsEmpty> {
        let Some(StackElement::Frame(popped_frame)) = self.frames.pop() else {
            return Err(StackIsEmpty);
        };

        if let Some(StackElement::Frame(new_top_frame)) = self.frames.last_mut()
        {
            for value in popped_frame.operands.values() {
                new_top_frame.operands.push(value);
            }
        }

        Ok(())
    }

    pub fn define_binding(&mut self, name: String, value: impl Into<Value>) {
        let StackElement::Frame(frame) = self.frames.last_mut().unwrap();
        frame.bindings.insert(name, value.into());
    }

    pub fn push_operand(&mut self, operand: impl Into<Value>) {
        let StackElement::Frame(frame) = self.frames.last_mut().unwrap();
        frame.operands.push(operand.into());
    }

    pub fn pop_operand(&mut self) -> Result<Value, PopOperandError> {
        let StackElement::Frame(frame) = self.frames.last_mut().unwrap();
        frame.operands.pop_any()
    }

    pub fn take_next_instruction(&mut self) -> Option<InstructionAddress> {
        let StackElement::Frame(frame) = self.frames.last_mut()?;
        Some(frame.take_next_instruction())
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum StackElement {
    Frame(StackFrame),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct StackFrame {
    pub next_instruction: InstructionAddress,
    pub bindings: Bindings,
    pub operands: Operands,
}

impl StackFrame {
    fn new(next_instruction: InstructionAddress) -> Self {
        Self {
            next_instruction,
            bindings: Bindings::default(),
            operands: Operands::default(),
        }
    }

    fn take_next_instruction(&mut self) -> InstructionAddress {
        let next = self.next_instruction;
        self.next_instruction.increment();
        next
    }
}

pub type Bindings = BTreeMap<String, Value>;

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    thiserror::Error,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum PushStackFrameError {
    #[error(transparent)]
    PopOperand(#[from] PopOperandError),

    #[error("Reached recursion limit")]
    Overflow,
}

#[derive(Debug)]
pub struct StackIsEmpty;
