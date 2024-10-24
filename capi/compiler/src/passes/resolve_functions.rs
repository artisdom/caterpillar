use std::collections::BTreeMap;

use crate::{
    code::{CallGraph, Fragment, Function, NamedFunctions},
    hash::Hash,
};

pub fn resolve_calls_to_user_defined_functions(
    named_functions: &mut NamedFunctions,
    call_graph: &CallGraph,
) {
    let mut resolved_hashes_by_name = BTreeMap::new();

    for (index, _) in call_graph.functions_from_leaves() {
        let function = named_functions
            .get_mut(index)
            .expect("Function referred to from call graph must exist.");

        resolve_calls_in_function(function, &mut resolved_hashes_by_name);

        let name = function.name.clone().expect(
            "Just compiled a named function; should have its name set.",
        );
        resolved_hashes_by_name.insert(name, Hash::new(function));
    }
}

fn resolve_calls_in_function(
    function: &mut Function,
    resolved_hashes_by_name: &mut BTreeMap<String, Hash<Function>>,
) {
    for branch in function.branches.values_mut() {
        for typed_fragment in branch.body.values_mut() {
            resolve_calls_in_fragment(
                &mut typed_fragment.fragment,
                resolved_hashes_by_name,
            );
        }
    }
}

fn resolve_calls_in_fragment(
    fragment: &mut Fragment,
    resolved_hashes_by_name: &mut BTreeMap<String, Hash<Function>>,
) {
    match fragment {
        Fragment::Function { function } => {
            resolve_calls_in_function(function, resolved_hashes_by_name);
        }
        Fragment::UnresolvedIdentifier {
            name,
            is_known_to_be_in_tail_position,
            is_known_to_be_call_to_user_defined_function,
        } => {
            // By the time we make it to this compiler pass, all expressions
            // that are in tail position should be known to be so.
            let is_in_tail_position = is_known_to_be_in_tail_position;

            if let Some(call) = is_known_to_be_call_to_user_defined_function {
                // By the time we make it to this compiler pass, all calls that
                // are recursive should be known to be so.
                let is_recursive_call_to_index =
                    call.is_known_to_be_recursive_call;

                if let Some(index) = is_recursive_call_to_index {
                    *fragment = Fragment::CallToUserDefinedFunctionRecursive {
                        index,
                        is_tail_call: *is_in_tail_position,
                    }
                } else {
                    let Some(hash) = resolved_hashes_by_name.get(name).copied()
                    else {
                        panic!(
                            "Resolving call to function `{name}`. Expecting \
                            called function to already be resolved when its \
                            caller is being resolved."
                        );
                    };

                    *fragment = Fragment::CallToUserDefinedFunction {
                        hash,
                        is_tail_call: *is_in_tail_position,
                    };
                }
            }
        }
        _ => {}
    }
}
