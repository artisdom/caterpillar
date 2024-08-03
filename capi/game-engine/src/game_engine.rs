use std::collections::VecDeque;

use capi_process::{Bytecode, Process, Value};

use crate::{host::GameEngineHost, input::Input, memory::Memory};

pub struct GameEngine {
    pub arguments: Vec<Value>,
    pub bytecode: Option<Bytecode>,
    pub process: Process<GameEngineHost>,
    pub memory: Memory,
    pub input: Input,
    pub random: VecDeque<i32>,
}

impl GameEngine {
    pub fn on_new_bytecode(&mut self, bytecode: Bytecode) {
        self.bytecode = Some(bytecode);
        self.reset();
    }

    pub fn reset(&mut self) {
        self.memory = Memory::default();
        self.process.reset(self.arguments.clone());
    }
}
