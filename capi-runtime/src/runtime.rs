use std::collections::VecDeque;

use crate::InstructionAddress;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub arguments: Vec<String>,
    pub instructions: VecDeque<InstructionAddress>,
}

impl Function {
    pub fn new(arguments: Vec<String>) -> Self {
        Self {
            arguments,
            instructions: VecDeque::new(),
        }
    }
}
