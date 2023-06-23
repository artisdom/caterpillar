use std::collections::{BTreeMap, BTreeSet};

use super::data_stack::Value;

#[derive(Default)]
pub struct Bindings {
    declarations: BTreeSet<String>,
    definitions: BTreeMap<String, Value>,
}

impl Bindings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, name: String) {
        self.declarations.insert(name);
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.definitions.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.definitions.get(name)
    }
}
