use capi_runtime::Instructions;

use crate::{
    fragments::{Fragments, NamedFunctions},
    host::Host,
    passes::{
        create_call_graph, detect_changes, determine_tail_positions,
        generate_fragments, generate_instructions, mark_recursive_calls, parse,
        resolve_identifiers, tokenize,
    },
    source_map::SourceMap,
};

/// # Entry point to the compiler API
#[derive(Default)]
pub struct Compiler {
    old_functions: Option<NamedFunctions>,
    instructions: Instructions,
    source_map: SourceMap,
}

impl Compiler {
    /// # Compile the provided source code
    pub fn compile<H: Host>(
        &mut self,
        source: &str,
    ) -> (Fragments, Instructions, SourceMap) {
        let tokens = tokenize(source);
        let mut functions = parse(tokens);
        determine_tail_positions(&mut functions);
        resolve_identifiers::<H>(&mut functions);
        let (mut functions, call_graph) = create_call_graph(functions);
        mark_recursive_calls(&mut functions, &call_graph);

        let fragments = generate_fragments(functions, call_graph);
        let changes = detect_changes(
            self.old_functions.take(),
            &fragments.named_functions,
        );

        self.old_functions = Some(fragments.named_functions.clone());

        generate_instructions(
            &fragments,
            &changes,
            &mut self.instructions,
            &mut self.source_map,
        );

        (
            fragments,
            self.instructions.clone(),
            self.source_map.clone(),
        )
    }
}
