use crate::Value;

#[derive(
    Clone, Debug, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize,
)]
pub struct Operands {
    values: Vec<Value>,
}

impl Operands {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.values.push(value.into());
    }

    pub fn pop(&mut self) -> Result<Value, PopOperandError> {
        self.values.pop().ok_or(PopOperandError::MissingOperand)
    }

    pub fn values(&self) -> impl Iterator<Item = Value> + '_ {
        self.values.iter().copied()
    }
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    thiserror::Error,
    serde::Deserialize,
    serde::Serialize,
)]
#[error("Missing operand")]
pub enum PopOperandError {
    MissingOperand,
}
