use capi_process::Process;
use capi_protocol::{
    host::GameEngineHost,
    memory::Memory,
    updates::{SourceCode, Update},
};

use super::{ActiveFunctions, Debugger};

#[derive(Debug, Default)]
pub struct RemoteProcess {
    pub source_code: Option<SourceCode>,
    pub process: Option<Process<GameEngineHost>>,
    pub memory: Option<Memory>,
}

impl RemoteProcess {
    pub fn on_update(&mut self, update: Update) {
        match update {
            Update::Memory { memory } => {
                self.memory = Some(memory);
            }
            Update::Process(process) => {
                self.process = Some(process);
            }
            Update::SourceCode(source_code) => {
                self.source_code = Some(source_code);
            }
        }
    }

    pub fn to_debugger(&self) -> Debugger {
        let active_functions = ActiveFunctions::new(
            self.source_code.as_ref(),
            self.process.as_ref(),
        );
        let operands = self
            .process
            .as_ref()
            .and_then(|process| process.stack().operands().cloned());
        let memory = self.memory.clone();

        Debugger {
            active_functions,
            operands,
            memory,
        }
    }
}
