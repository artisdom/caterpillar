use capi_vm::opcode;

pub fn assemble(assembly: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut bytecode = Vec::new();

    let mut instructions = assembly.split_whitespace();

    while let Some(instruction) = instructions.next() {
        match instruction {
            "terminate" => bytecode.push(opcode::TERMINATE),
            instruction => {
                return Err(AssemblerError::UnknownInstruction {
                    name: instruction.into(),
                })
            }
        }
    }

    Ok(bytecode)
}

#[derive(Debug, thiserror::Error)]
pub enum AssemblerError {
    #[error("Unknown instruction: `{name}`")]
    UnknownInstruction { name: String },
}

#[cfg(test)]
mod tests {
    use capi_vm::Evaluator;

    #[test]
    fn terminate() {
        assemble("terminate", []);
        // This should not run forever, nor cause any kind of panic.
    }

    fn assemble<const D: usize>(assembly: &str, mut data: [u8; D]) -> [u8; D] {
        let bytecode = super::assemble(assembly).unwrap();

        let mut evaluator = Evaluator::new(&data);
        evaluator.evaluate(&bytecode, &mut data);

        data
    }
}
