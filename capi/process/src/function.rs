use crate::instructions::InstructionAddr;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub arguments: Vec<String>,
    pub start: InstructionAddr,
}
