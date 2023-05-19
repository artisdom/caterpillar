pub struct DataStack {
    values: Vec<bool>,
}

impl DataStack {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: bool) {
        self.values.push(value)
    }

    pub fn pop_bool(&mut self) -> Result<bool, DataStackError> {
        self.values.pop().ok_or(DataStackError::PopFromEmptyStack)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DataStackError {
    #[error("Tried to pop value from empty stack")]
    PopFromEmptyStack,
}
