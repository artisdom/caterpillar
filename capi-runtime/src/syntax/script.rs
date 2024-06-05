use super::{Functions, SyntaxBuilder};

#[derive(Default)]
pub struct Script {
    pub functions: Functions,
}

impl Script {
    pub fn function<'r>(
        &mut self,
        name: &str,
        _args: impl IntoIterator<Item = &'r str>,
        f: impl FnOnce(&mut SyntaxBuilder),
    ) {
        self.functions.define(name, f)
    }
}
