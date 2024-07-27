use super::{Function, SyntaxBuilder};

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Script {
    pub functions: Vec<Function>,
}

impl Script {
    pub fn function<'r>(
        &mut self,
        name: &str,
        args: impl IntoIterator<Item = &'r str>,
        f: impl FnOnce(&mut SyntaxBuilder),
    ) -> &mut Self {
        let mut body = Vec::new();
        f(&mut SyntaxBuilder::new(&mut body));

        self.functions.push(Function {
            name: name.to_string(),
            args: args.into_iter().map(String::from).collect(),
            body,
        });

        self
    }
}
