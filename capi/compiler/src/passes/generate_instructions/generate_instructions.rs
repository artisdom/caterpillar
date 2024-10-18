use std::collections::BTreeMap;

use capi_runtime::{Effect, Instruction, InstructionAddress, Instructions};

use crate::{
    code::{CallGraph, Changes, Function, NamedFunctions},
    compiler::CallInstructionsByCallee,
    hash::Hash,
    source_map::SourceMap,
};

use super::{
    compile_function::{compile_call_to_function, CallToFunction},
    compile_named_functions::compile_named_functions,
};

pub fn generate_instructions(
    named_functions: &NamedFunctions,
    call_graph: &CallGraph,
    changes: &Changes,
    instructions: &mut Instructions,
    call_instructions_by_callee: &mut CallInstructionsByCallee,
    source_map: &mut SourceMap,
) {
    // The placeholder call into `main` is created unconditionally, regardless
    // of whether this is a fresh build and we actually need to do that, or if
    // we already have an active runtime and are just compiling changes.
    //
    // I don't think this has any adverse effects, except creating junk
    // instructions that increase the code size. And I don't want to fix that,
    // until we have infrastructure in place that would measure the code size
    // and actually show the impact of those changes.
    //
    // Otherwise, we'll just complicate the code with unclear benefit, and no
    // means to track whether simplifications are beneficial or not.
    let call_to_main = create_placeholder_for_call_to_main(instructions);

    let mut functions = compile_named_functions(
        named_functions,
        changes,
        call_graph,
        instructions,
        source_map,
        call_instructions_by_callee,
    );
    compile_call_to_main(
        call_to_main,
        named_functions,
        instructions,
        &mut functions,
    );
}

fn create_placeholder_for_call_to_main(
    instructions: &mut Instructions,
) -> InstructionAddress {
    // If there's no `main` function, this instruction won't get replaced later.
    // That would be a result of invalid code (valid code would provide a `main`
    // function), so an instruction generating the `BuildError` effect is an
    // appropriate placeholder.
    instructions.push(Instruction::TriggerEffect {
        effect: Effect::BuildError,
    })
}

fn compile_call_to_main(
    call_to_main: InstructionAddress,
    named_functions: &NamedFunctions,
    instructions: &mut Instructions,
    functions: &mut BTreeMap<Hash<Function>, capi_runtime::Function>,
) {
    if let Some(main) = named_functions.find_by_name("main") {
        compile_call_to_function(
            &Hash::new(&main),
            &CallToFunction {
                address: call_to_main,
                is_tail_call: true,
            },
            functions,
            instructions,
        );
    }
}
