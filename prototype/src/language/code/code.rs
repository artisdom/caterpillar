use std::collections::BTreeSet;

use super::{Body, Fragment, FragmentId, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: Body,
    pub errors: BTreeSet<FragmentId>,
}

impl Code {
    pub fn entry(&self) -> Option<FragmentId> {
        self.root.inner.first().copied()
    }

    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn push(&mut self, fragment: Fragment) -> FragmentId {
        let id = self.fragments.insert(fragment);
        self.root.inner.push(id);
        id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: usize, argument: FragmentId },
    LiteralValue { value: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
