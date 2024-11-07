use anyhow::anyhow;
use capi_compiler::{code::ExpressionLocation, CompilerOutput};
use capi_runtime::{Instruction, InstructionAddress};

#[derive(Clone, Debug, Default)]
pub struct DebugCode {
    pub inner: Option<CompilerOutput>,
}

impl DebugCode {
    pub fn get(&self) -> anyhow::Result<&CompilerOutput> {
        self.inner
            .as_ref()
            .ok_or_else(|| anyhow!("Code is not available yet."))
    }

    pub fn expression_to_instruction(
        &self,
        expression: &ExpressionLocation,
    ) -> anyhow::Result<InstructionAddress> {
        let code = self.get()?;
        code.source_map
            .fragment_to_instructions(expression)
            .first()
            .copied()
            .ok_or_else(|| anyhow!("Fragment does not map to instruction."))
    }

    pub fn instruction(
        &self,
        address: &InstructionAddress,
    ) -> anyhow::Result<&Instruction> {
        let code = self.get()?;
        code.instructions.get(address).ok_or_else(|| {
            anyhow!("Could not find instruction at `{address}`.")
        })
    }
}
