use std::collections::{btree_map, BTreeMap};

use super::{pipeline::c_analyzer::Expressions, Evaluator, EvaluatorError};

#[derive(Debug, Default)]
pub struct Functions {
    definitions: BTreeMap<String, Function>,
}

impl Functions {
    pub fn new() -> Functions {
        Self::default()
    }

    pub fn define(&mut self, module: Module, name: String, body: Expressions) {
        let module = module.name();
        let function = Function {
            module,
            body: FunctionBody::UserDefined { body },
        };
        self.definitions.insert(name, function);
    }

    pub fn define_intrinsic(
        &mut self,
        module: Module,
        name: String,
        body: IntrinsicBody,
    ) {
        let module = module.name();
        let function = Function {
            module,
            body: FunctionBody::Intrinsic { body },
        };
        self.definitions.insert(name, function);
    }

    pub fn is_declared(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<Function> {
        self.definitions.get(name).cloned()
    }
}

impl IntoIterator for Functions {
    type Item = (String, Function);
    type IntoIter = btree_map::IntoIter<String, Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.definitions.into_iter()
    }
}

impl<'a> IntoIterator for &'a Functions {
    type Item = (&'a String, &'a Function);
    type IntoIter = btree_map::Iter<'a, String, Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.definitions.iter()
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub module: String,
    pub body: FunctionBody,
}

#[derive(Clone, Debug)]
pub enum FunctionBody {
    Intrinsic { body: IntrinsicBody },
    UserDefined { body: Expressions },
}

pub type IntrinsicBody = fn(&mut Evaluator) -> Result<(), EvaluatorError>;

#[derive(Clone, Copy)]
pub struct Module<'r> {
    inner: Option<&'r str>,
}

impl<'r> Module<'r> {
    pub fn none() -> Self {
        Self { inner: None }
    }

    pub fn some(s: &'r str) -> Self {
        Self { inner: Some(s) }
    }

    pub fn name(&self) -> String {
        self.inner.unwrap_or("<root>").into()
    }
}
