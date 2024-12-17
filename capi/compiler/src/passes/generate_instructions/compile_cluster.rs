use std::collections::{BTreeMap, VecDeque};

use capi_runtime::InstructionAddress;

use crate::code::{
    syntax::{FunctionLocation, Located, SyntaxTree},
    DependencyCluster,
};

use super::{
    compile_function::{
        compile_call_to_function, compile_definition_of_local_function,
        compile_function, CallToFunction, FunctionToCompile,
    },
    compile_functions::FunctionsContext,
};

pub struct ClusterContext {
    /// # The queue of functions to compile in the cluster
    ///
    /// This is initially seeded by the named functions in the cluster that are
    /// new or have been updated. But any anonymous functions encountered while
    /// compiling those, will be added later.
    pub queue_of_functions_to_compile: VecDeque<FunctionToCompile>,

    /// # Recursive calls within the cluster that need to be replaced
    ///
    /// When a recursive call is encountered, not all branches of the callee
    /// (which might be the calling function itself, or another function in the
    /// same cluster) might be compiled yet. But they're needed to compile the
    /// call.
    ///
    /// So instead of compiling the call right then and there, a placeholder
    /// instruction is emitted. An entry is also added to this map, so the
    /// placeholder instruction can be replaced with the real call, once all
    /// functions have been compiled.
    pub recursive_calls_by_callee:
        BTreeMap<FunctionLocation, Vec<CallToFunction>>,

    /// # Recursive local functions within the cluster that need to be replaced
    ///
    /// When a recursive local function is encountered, it might not have been
    /// compiled yet. To deal with this, we generate a placeholder that needs to
    /// be replaced later. This map tracks those necessary replacements.
    pub recursive_local_function_definitions_by_local_function:
        BTreeMap<FunctionLocation, InstructionAddress>,
}

pub fn compile_cluster(
    cluster: &DependencyCluster,
    functions_context: &mut FunctionsContext,
) {
    let mut context = ClusterContext {
        queue_of_functions_to_compile: VecDeque::new(),
        recursive_calls_by_callee: BTreeMap::new(),
        recursive_local_function_definitions_by_local_function: BTreeMap::new(),
    };

    seed_queue_of_functions_to_compile(
        &mut context.queue_of_functions_to_compile,
        cluster,
        functions_context.syntax_tree,
    );

    while let Some(function_to_compile) =
        context.queue_of_functions_to_compile.pop_front()
    {
        let runtime_function = compile_function(
            Located {
                fragment: &function_to_compile.function,
                location: function_to_compile.location.clone(),
            },
            &mut context,
            functions_context,
        );

        functions_context
            .compiled_functions_by_location
            .insert(function_to_compile.location, runtime_function);
    }

    for (callee, calls) in context.recursive_calls_by_callee {
        for call in calls {
            compile_call_to_function(
                &callee,
                call,
                functions_context.compiled_functions_by_location,
                functions_context.instructions,
            );
        }
    }
    for (local_function, address) in
        context.recursive_local_function_definitions_by_local_function
    {
        compile_definition_of_local_function(
            local_function,
            address,
            functions_context,
        );
    }
}

fn seed_queue_of_functions_to_compile(
    queue_of_functions_to_compile: &mut VecDeque<FunctionToCompile>,
    cluster: &DependencyCluster,
    syntax_tree: &SyntaxTree,
) {
    let functions_in_cluster_to_compile =
        cluster
            .functions(syntax_tree)
            .map(|function| FunctionToCompile {
                function: function.fragment.clone(),
                location: function.location,
            });
    queue_of_functions_to_compile.extend(functions_in_cluster_to_compile);
}
