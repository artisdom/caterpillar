use std::collections::BTreeMap;

use super::code::{HostFunction, Signature};

pub struct Host {
    pub functions: BTreeMap<String, Signature>,
}

impl Host {
    pub fn functions(
        &self,
    ) -> impl Iterator<Item = (&String, HostFunction)> + '_ {
        self.functions
            .iter()
            .map(|(name, &signature)| (name, HostFunction { signature }))
    }

    pub fn function_by_name(&self, name: &str) -> Option<HostFunction> {
        self.functions()
            .find_map(|(n, function)| (n == name).then_some(function))
    }
}
