use std::collections::BTreeMap;

use crate::{
    instructions::{Instruction, Instructions},
    runtime, InstructionAddress,
};

use super::symbols::Symbols;

#[derive(
    Clone, Debug, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize,
)]
pub struct Code {
    pub symbols: Symbols,
    pub instructions: Instructions,
    pub functions: BTreeMap<String, runtime::Function>,
}

impl Code {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_address(&self) -> InstructionAddress {
        self.instructions.next_address()
    }

    pub fn push(&mut self, instruction: Instruction) -> InstructionAddress {
        self.instructions.push(instruction)
    }
}
