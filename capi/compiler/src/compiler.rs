use capi_runtime::Instructions;

use crate::{
    fragments::Fragments,
    host::Host,
    passes::{
        determine_tail_positions, generate_fragments, generate_instructions,
        group_into_clusters, mark_recursive_calls, parse, resolve_identifiers,
        tokenize,
    },
    source_map::SourceMap,
};

/// # Entry point to the compiler API
pub struct Compiler {}

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
        let mut clusters = group_into_clusters(functions);
        mark_recursive_calls(&mut clusters);
        let fragments = generate_fragments(clusters);
        let (instructions, source_map) =
            generate_instructions(fragments.clone());

        (fragments, instructions, source_map)
    }
}
