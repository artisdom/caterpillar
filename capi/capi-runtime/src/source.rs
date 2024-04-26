use crate::{
    code::Code, compiler::compile, source_map::SourceMap, syntax::Syntax,
    Function, Functions, InstructionAddress, Program,
};

#[derive(Default)]
pub struct Source {
    pub functions: Functions,
}

impl Source {
    pub fn define(&mut self, name: &str, f: impl FnOnce(&mut Syntax)) {
        self.functions.define(name, f)
    }

    pub fn compile(self, entry: &str) -> Program {
        let mut code = Code::new();
        let mut source_map = SourceMap::default();

        for Function { name, syntax } in &self.functions.inner {
            compile(
                name.clone(),
                syntax.clone(),
                &self.functions.names,
                &mut code,
                &mut source_map,
            );
        }

        let entry = code.symbols.resolve(entry);

        let mut program = Program {
            functions: self.functions,
            source_map,
            ..Program::default()
        };
        program.evaluator.update(code, InstructionAddress(entry));
        program.entry = entry;

        program
    }
}
