use std::collections::BTreeMap;

use crate::code::{CallGraph, Expression, Function, Functions, Hash};

pub fn resolve_calls_to_user_defined_functions(
    functions: &mut Functions,
    call_graph: &CallGraph,
) {
    let mut resolved_hashes_by_name = BTreeMap::new();

    for (index, _) in call_graph.functions_from_leaves() {
        let function = functions
            .named
            .get_mut(index)
            .expect("Function referred to from call graph must exist.");

        resolve_calls_in_function(
            &mut function.inner,
            &mut resolved_hashes_by_name,
        );

        resolved_hashes_by_name
            .insert(function.name.clone(), Hash::new(&function.inner));
    }
}

fn resolve_calls_in_function(
    function: &mut Function,
    resolved_hashes_by_name: &mut BTreeMap<String, Hash<Function>>,
) {
    for branch in function.branches.values_mut() {
        for expression in branch.body.values_mut() {
            resolve_calls_in_expression(expression, resolved_hashes_by_name);
        }
    }
}

fn resolve_calls_in_expression(
    expression: &mut Expression,
    resolved_hashes_by_name: &mut BTreeMap<String, Hash<Function>>,
) {
    match expression {
        Expression::LiteralFunction { function } => {
            resolve_calls_in_function(function, resolved_hashes_by_name);
        }
        Expression::UnresolvedIdentifier {
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
                    *expression =
                        Expression::CallToUserDefinedFunctionRecursive {
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

                    *expression = Expression::CallToUserDefinedFunction {
                        hash,
                        is_tail_call: *is_in_tail_position,
                    };
                }
            }
        }
        _ => {}
    }
}
