use std::collections::{BTreeSet, VecDeque};

use crate::Value;

#[derive(
    Clone, Debug, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize,
)]
pub struct Instructions {
    inner: InstructionsInner,
}

impl Instructions {
    pub fn push(&mut self, instruction: Instruction) -> InstructionAddress {
        let addr = InstructionAddress {
            index: self.inner.len().try_into().unwrap(),
        };
        self.inner.push_back((addr, instruction));
        addr
    }

    pub fn get(&self, addr: &InstructionAddress) -> Option<&Instruction> {
        let (stored_addr, instruction) = self.inner.get(addr.to_usize())?;
        assert_eq!(addr, stored_addr);
        Some(instruction)
    }

    pub fn replace(
        &mut self,
        addr: InstructionAddress,
        instruction: Instruction,
    ) {
        let (stored_addr, stored_instruction) =
            self.inner.get_mut(addr.to_usize()).unwrap();
        assert_eq!(addr, *stored_addr);
        *stored_instruction = instruction;
    }
}

impl<'r> IntoIterator for &'r Instructions {
    type Item = <&'r InstructionsInner as IntoIterator>::Item;
    type IntoIter = <&'r InstructionsInner as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

type InstructionsInner = VecDeque<(InstructionAddress, Instruction)>;

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct InstructionAddress {
    pub index: u32,
}

impl InstructionAddress {
    pub fn increment(&mut self) {
        self.index += 1;
    }

    fn to_usize(self) -> usize {
        self.index
            .try_into()
            .expect("Expected `usize` to cover full range of `u32`")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Instruction {
    BindingEvaluate {
        name: String,
    },
    BindingsDefine {
        names: Vec<String>,
    },
    CallBuiltin {
        name: String,
    },
    CallFunction {
        address: InstructionAddress,
    },
    MakeClosure {
        addr: InstructionAddress,
        environment: BTreeSet<String>,
    },
    Push {
        value: Value,
    },
    Return,
    ReturnIfNonZero,
    ReturnIfZero,
    Panic,
}
