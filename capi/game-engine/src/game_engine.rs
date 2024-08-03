use capi_process::{Bytecode, Process, Value};

use crate::host::GameEngineHost;

pub struct GameEngine {
    pub arguments: Vec<Value>,
    pub bytecode: Option<Bytecode>,
    pub process: Process<GameEngineHost>,
}

impl GameEngine {
    pub fn on_new_bytecode(&mut self, bytecode: Bytecode) {
        self.process.reset(self.arguments.clone());
        self.bytecode = Some(bytecode);
    }
}
