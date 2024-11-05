use std::{
    collections::{btree_map::Entry, BTreeMap, VecDeque},
    iter, vec,
};

use crate::{
    code::{
        Branch, BranchLocation, CallGraph, Cluster, ConcreteSignature,
        Fragment, FragmentLocation, FunctionLocation, Index, NamedFunctions,
        Pattern, Signature, Type, Types,
    },
    host::Host,
    intrinsics::IntrinsicFunction,
};

pub fn infer_types(
    named_functions: &NamedFunctions,
    call_graph: &CallGraph,
    host: &impl Host,
) -> Types {
    let mut types = Types::default();

    for cluster in call_graph.clusters_from_leaves() {
        // For every cluster, we create a queue of the branches within that
        // cluster. This queue is not processed in its initial order. Processing
        // of a branch can be paused, and other branches can be processed first.
        //
        // The purpose of this is to provide full support for inferring the type
        // of recursive calls within a cluster. To infer such a type, another
        // branch might need to be processed first. In that case, we can queue
        // other branches of the called function first, before continuing with
        // the current branch.
        //
        // Or, in cases of endless recursion (mutual or self), the call can be
        // diverging. Using the queue, we can detect this by watching out for
        // repeated processing of the same branch.
        //
        // This approach has drawbacks. It is complex, and it introduces the
        // risk of an endless loop, if implemented incorrectly. You might think
        // that another approach, based on a call graph of functions within a
        // cluster, might be better.
        //
        // This could very well be. I went down this road, and kept running into
        // problems. Looking back, I might have ended this exploration too
        // early, as I no longer think the problems I ran into were intractable.
        //
        // Or I might have forgotten a critical details that actually _did_ make
        // those problems intractable. Either way, the queue approach works for
        // now.
        let mut queue = VecDeque::new();

        for index in cluster.functions.values() {
            let function = named_functions
                .find_by_index(index)
                .expect("Function referred to from call graph must exist.");

            for (&index, branch) in function.branches.iter() {
                let location = BranchLocation {
                    parent: Box::new(function.location().clone()),
                    index,
                };

                let environment = BTreeMap::new();
                queue.push_back(QueueItem::new(
                    branch,
                    location,
                    function.location(),
                    &environment,
                    &mut types,
                ));
            }
        }

        while let Some(queue_item) = queue.pop_front() {
            infer_types_in_branch(
                queue_item,
                cluster,
                named_functions,
                host,
                &mut queue,
                &mut types,
            );
        }
    }

    types
}

fn infer_types_in_branch(
    mut queue_item: QueueItem,
    cluster: &Cluster,
    named_functions: &NamedFunctions,
    host: &impl Host,
    queue: &mut BranchQueue,
    types: &mut Types,
) {
    while let Some((index, fragment)) = queue_item.branch_body.peek() {
        let location = FragmentLocation {
            parent: Box::new(queue_item.branch_location.clone()),
            index: *index,
        };

        let inference = infer_type_of_fragment(
            fragment,
            &location,
            cluster,
            named_functions,
            &queue_item.bindings,
            host,
            &mut queue_item.stack,
            types,
        );

        if let Some(inference) = inference {
            let signature = match inference {
                FragmentInference::Inferred { signature } => signature,
                FragmentInference::NeedToInferMoreBranchesFirst {
                    queue_items,
                } => {
                    // The fragment is a function literal. We need to infer the
                    // types of its branches before we can proceed.
                    //
                    // Let's schedule those to be inferred next, and pick up the
                    // inference of the current branch right after.
                    for queue_item in
                        [queue_item].into_iter().chain(queue_items)
                    {
                        queue.push_front(queue_item);
                    }

                    // Abort the inference of the current branch. Since we used
                    // `peek` above, we'll resume with the current fragment,
                    // once this branch is up again.
                    return;
                }
            };

            for &output in &signature.outputs {
                queue_item.stack.push(output);
            }
            types.of_fragments.insert(location, signature);
        }

        queue_item.branch_body.next().expect(
            "Just used `peek` to confirm there is an item in the queue; it \
            must still be there.",
        );
    }

    let signature = Signature {
        inputs: queue_item.parameters.into_iter().collect(),
        outputs: queue_item.stack,
    };

    types
        .of_branches
        .insert(queue_item.branch_location.clone(), signature.clone());

    types
        .of_branches
        .insert(queue_item.branch_location, signature.clone());

    match types.of_functions.entry(queue_item.function_location) {
        Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(signature);
        }
        Entry::Occupied(_occupied_entry) => {
            // If this isn't the first branch we're looking at, there already is
            // a function signature. We should compare that to the new branch
            // signature and make sure they're equal.
            //
            // As of this writing, type inference is only partially implemented
            // though, and as a result, this would trigger false positives all
            // the time.
            //
            // Let's just ignore any mismatches, for the time being.
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn infer_type_of_fragment(
    fragment: &Fragment,
    location: &FragmentLocation,
    cluster: &Cluster,
    named_functions: &NamedFunctions,
    bindings: &BTreeMap<String, Index<Type>>,
    host: &impl Host,
    stack: &mut Vec<Index<Type>>,
    types: &mut Types,
) -> Option<FragmentInference> {
    assert!(
        !types.of_fragments.contains_key(location),
        "Encountered a fragment whose type signature has already been \
        inferred:\n\
        {fragment:#?}\n\
        \n\
        The type of a fragment should be inferred only once."
    );

    let signature = match fragment {
        Fragment::Binding { name, .. } => {
            let Some(type_) = bindings.get(name).copied() else {
                unreachable!(
                    "Can't find binding `{name}` in `bindings`, but \n\
                    \n\
                    a) all local bindings are added to `bindings` above, and \n\
                    b) if we encounter a `Fragment::Binding`, as we are here, \
                       that was put there by an earlier compiler pass \
                       _because_ it resolves to a local binding."
                );
            };

            Signature {
                inputs: vec![],
                outputs: vec![type_],
            }
        }
        Fragment::CallToHostFunction { number } => {
            let signature = host
                .function_by_number(*number)
                .expect(
                    "Call to host function has already been resolved. Must \
                    refer to a host function.",
                )
                .signature();

            handle_concrete_signature(signature, stack, types)
        }
        Fragment::CallToIntrinsicFunction { intrinsic, .. } => {
            match (intrinsic, intrinsic.signature()) {
                (_, Some(signature)) => {
                    handle_concrete_signature(signature, stack, types)
                }
                (IntrinsicFunction::Eval, None) => {
                    let function = stack
                        .last()
                        .expect("`eval` takes at least one argument");
                    let function = types
                        .inner
                        .get(function)
                        .expect("Type referred to from stack must exist.");

                    let Type::Function { signature } = function else {
                        panic!("`eval` expects function on stack");
                    };

                    // `eval` has the same signature as the function it
                    // consumes, except that it consumes that function in
                    // addition.
                    let mut signature = signature.clone();
                    signature.inputs.push(types.inner.push(Type::Function {
                        signature: signature.clone(),
                    }));

                    signature
                }
                (intrinsic, signature) => {
                    unreachable!(
                        "Invalid combination of intrinsic (`{intrinsic:?}`) \
                        and signature (`{signature:?}`"
                    );
                }
            }
        }
        Fragment::CallToUserDefinedFunction { hash, .. } => {
            let function = named_functions
                .find_by_hash(hash)
                .expect("Function referred to by resolved call must exist.");

            types
                .of_functions
                .get(&function.location())
                .expect(
                    "This compiler pass infers function types by call graph, \
                    from the leaves up. Thus, the signature of a function must \
                    have been inferred, before a call for it is being \
                    inferred.",
                )
                .clone()
        }
        Fragment::CallToUserDefinedFunctionRecursive { index, .. } => {
            // Type inference of recursive function calls is not fully
            // implemented yet. This is just a starting point.

            let index = *cluster.functions.get(index).expect(
                "Function referred to by recursive call must exist in cluster.",
            );
            let location = FunctionLocation::from(index);

            match types.of_functions.get(&location).cloned() {
                Some(signature) => signature,
                None => {
                    let inputs = {
                        let inputs = stack.clone();
                        stack.clear();
                        inputs
                    };
                    let outputs = {
                        let empty = types.inner.push(Type::Empty);
                        vec![empty]
                    };

                    Signature { inputs, outputs }
                }
            }
        }
        Fragment::Comment { .. } => {
            // Comments have no bearing on type inference.
            return None;
        }
        Fragment::Function { function } => {
            let function_location = FunctionLocation::AnonymousFunction {
                location: location.clone(),
            };

            let Some(signature) =
                types.of_functions.get(&function_location).cloned()
            else {
                let mut queue_items = Vec::new();

                for (&index, branch) in function.branches.iter() {
                    let branch_location = BranchLocation {
                        parent: Box::new(function_location.clone()),
                        index,
                    };

                    queue_items.push(QueueItem::new(
                        branch,
                        branch_location.clone(),
                        function_location.clone(),
                        bindings,
                        types,
                    ));
                }

                return Some(FragmentInference::NeedToInferMoreBranchesFirst {
                    queue_items,
                });
            };

            let type_ = types.inner.push(Type::Function { signature });

            Signature {
                inputs: vec![],
                outputs: vec![type_],
            }
        }
        Fragment::UnresolvedIdentifier { .. } => {
            // There nothing we can do here, really. This has already been
            // identified as a problem.
            return None;
        }
        Fragment::Value(_) => Signature {
            inputs: vec![],
            outputs: vec![types.inner.push(Type::Number)],
        },
    };

    Some(FragmentInference::Inferred { signature })
}

fn handle_concrete_signature(
    ConcreteSignature { inputs, outputs }: ConcreteSignature,
    stack: &mut Vec<Index<Type>>,
    types: &mut Types,
) -> Signature {
    let mut signature = Signature {
        inputs: Vec::new(),
        outputs: Vec::new(),
    };

    for input in inputs.iter().rev() {
        if let Some(index) = stack.pop() {
            let type_ = types
                .inner
                .get_mut(&index)
                .expect("Type that is referenced from stack must exist.");
            *type_ = input.clone();
        } else {
            // It looks like we don't have enough types on the stack for the
            // number of inputs this fragment has.
            //
            // Typically, this would be an error in type checking, but since the
            // inference is not fully implemented yet, it could also be due to
            // that.
            //
            // Let's ignore it for now.
        }
    }

    for input in inputs {
        let index = types.inner.push(input.clone());
        signature.inputs.push(index);
    }
    for output in outputs {
        let index = types.inner.push(output.clone());
        signature.outputs.push(index);
    }

    signature
}

type BranchQueue = VecDeque<QueueItem>;

struct QueueItem {
    branch_body: iter::Peekable<vec::IntoIter<(Index<Fragment>, Fragment)>>,
    branch_location: BranchLocation,
    function_location: FunctionLocation,
    parameters: Vec<Index<Type>>,
    bindings: BTreeMap<String, Index<Type>>,
    stack: Vec<Index<Type>>,
}

impl QueueItem {
    fn new(
        branch: &Branch,
        branch_location: BranchLocation,
        function_location: FunctionLocation,
        environment: &BTreeMap<String, Index<Type>>,
        types: &mut Types,
    ) -> Self {
        let mut parameters = Vec::new();
        let mut bindings = environment.clone();

        for pattern in branch.parameters.iter() {
            let type_ = match pattern {
                Pattern::Identifier { name } => {
                    let type_ = types.inner.push(Type::Unknown);
                    bindings.insert(name.clone(), type_);
                    type_
                }
                Pattern::Literal { .. } => types.inner.push(Type::Number),
            };

            parameters.push(type_);
        }

        Self {
            branch_body: branch
                .body
                .iter()
                .map(|(index, fragment)| (*index, fragment.clone()))
                .collect::<Vec<_>>()
                .into_iter()
                .peekable(),
            branch_location,
            function_location,
            parameters,
            bindings,
            stack: Vec::new(),
        }
    }
}

enum FragmentInference {
    Inferred { signature: Signature },
    NeedToInferMoreBranchesFirst { queue_items: Vec<QueueItem> },
}

#[cfg(test)]
mod tests {
    use crate::{
        code::{ConcreteSignature, NamedFunctions, Type, Types},
        host::{Host, HostFunction},
        passes::{
            build_call_graph, mark_recursive_calls, parse,
            resolve_calls_to_user_defined_functions, resolve_most_identifiers,
            tokenize,
        },
    };

    #[test]
    fn infer_fragment_signatures_based_on_host_function() {
        let (named_functions, types) = type_fragments(
            r"
                f: fn
                    \ n ->
                        n number_to_nothing
                end
            ",
        );

        let mut fragments = named_functions
            .find_by_name("f")
            .unwrap()
            .find_single_branch()
            .unwrap()
            .body()
            .map(|fragment| {
                types
                    .of_fragments
                    .get(fragment.location())
                    .unwrap()
                    .to_concrete_signature(&types)
                    .unwrap()
            });

        let n = fragments.next().unwrap();
        let host_fn = fragments.next().unwrap();

        use Type::*;
        assert_eq!(n, ConcreteSignature::from(([], [Number])));
        assert_eq!(host_fn, ConcreteSignature::from(([Number], [])));
    }

    #[test]
    fn infer_type_of_function_literal() {
        let (named_functions, types) = type_fragments(
            r"
                f: fn
                    \ ->
                        fn
                            \ 0 ->
                                0
                        end
                end
            ",
        );

        let mut fragments = named_functions
            .find_by_name("f")
            .unwrap()
            .find_single_branch()
            .unwrap()
            .body()
            .map(|fragment| {
                types
                    .of_fragments
                    .get(fragment.location())
                    .unwrap()
                    .to_concrete_signature(&types)
                    .unwrap()
            });

        let function = fragments.next().unwrap();

        assert_eq!(function.inputs, []);

        let [Type::Function { signature }] = &function.outputs.as_slice()
        else {
            panic!("Unexpected outputs: {:?}", function.outputs);
        };
        let signature = signature.to_concrete_signature(&types).unwrap();

        use Type::*;
        assert_eq!(signature, ConcreteSignature::from(([Number], [Number])));
    }

    #[test]
    #[should_panic] // known bug; not currently tracked in an issue
    fn infer_type_of_function_based_on_most_specific_branch() {
        let (named_functions_a, types_a) = type_fragments(
            r"
                f: fn
                    \ 0 ->
                        0

                    \ n ->
                        n
                end
            ",
        );
        let (named_functions_b, types_b) = type_fragments(
            r"
                f: fn
                    \ n ->
                        n

                    \ 0 ->
                        0
                end
            ",
        );

        check(&named_functions_a, &types_a);
        check(&named_functions_b, &types_b);

        fn check(named_functions: &NamedFunctions, types: &Types) {
            let f = named_functions
                .find_by_name("f")
                .map(|function| {
                    types
                        .of_functions
                        .get(&function.location())
                        .unwrap()
                        .to_concrete_signature(types)
                        .unwrap()
                })
                .unwrap();

            use Type::*;
            assert_eq!(f, ConcreteSignature::from(([Number], [Number])),);
        }
    }

    #[test]
    fn infer_signatures_of_branch_and_function() {
        let (named_functions, types) = type_fragments(
            r"
                f: fn
                    \ a, b, 0 ->
                        a number_to_nothing
                        b
                end
            ",
        );

        let branch = named_functions
            .find_by_name("f")
            .unwrap()
            .find_single_branch()
            .map(|branch| {
                types
                    .of_branches
                    .get(branch.location())
                    .unwrap()
                    .to_concrete_signature(&types)
                    .unwrap()
            })
            .unwrap();
        let function = named_functions
            .find_by_name("f")
            .map(|function| {
                types
                    .of_functions
                    .get(&function.location())
                    .unwrap()
                    .to_concrete_signature(&types)
                    .unwrap()
            })
            .unwrap();

        use Type::*;
        let expected_signature =
            ConcreteSignature::from(([Number, Unknown, Number], [Unknown]));

        assert_eq!(branch, expected_signature);
        assert_eq!(function, expected_signature);
    }

    #[test]
    fn infer_self_recursive_function_as_empty() {
        let (named_functions, types) = type_fragments(
            r"
                f: fn
                    \ a, b, 0 ->
                        a number_to_nothing
                        0 b 0 f
                end
            ",
        );
        dbg!(&named_functions);

        let f = named_functions
            .find_by_name("f")
            .map(|function| {
                types
                    .of_functions
                    .get(&function.location())
                    .unwrap()
                    .to_concrete_signature(&types)
                    .unwrap()
            })
            .unwrap();

        use Type::*;
        assert_eq!(
            f,
            ConcreteSignature::from(([Number, Unknown, Number], [Empty])),
        );
    }

    #[test]
    fn infer_self_recursive_non_empty_function() {
        let (named_functions, types) = type_fragments(
            r"
                f: fn
                    \ a, b, 0 ->
                        a number_to_nothing
                        0

                    \ a, b, 1 ->
                        0 b 0 f
                end
            ",
        );

        check(&named_functions, &types);

        fn check(named_functions: &NamedFunctions, types: &Types) {
            let f = named_functions
                .find_by_name("f")
                .map(|function| {
                    types
                        .of_functions
                        .get(&function.location())
                        .unwrap()
                        .to_concrete_signature(types)
                        .unwrap()
                })
                .unwrap();

            use Type::*;
            assert_eq!(
                f,
                ConcreteSignature::from(([Number, Unknown, Number], [Number])),
            );
        }
    }

    fn type_fragments(source: &str) -> (NamedFunctions, Types) {
        let tokens = tokenize(source);
        let mut named_functions = parse(tokens);
        resolve_most_identifiers(&mut named_functions, &TestHost);
        let call_graph = build_call_graph(&named_functions);
        mark_recursive_calls(&mut named_functions, &call_graph);
        resolve_calls_to_user_defined_functions(
            &mut named_functions,
            &call_graph,
        );
        let types =
            super::infer_types(&named_functions, &call_graph, &TestHost);

        (named_functions, types)
    }

    struct TestHost;

    impl Host for TestHost {
        fn functions(&self) -> impl IntoIterator<Item = &dyn HostFunction> {
            [&NumberToNothing as &_]
        }
    }

    struct NumberToNothing;

    impl HostFunction for NumberToNothing {
        fn number(&self) -> u8 {
            0
        }

        fn name(&self) -> &'static str {
            "number_to_nothing"
        }

        fn signature(&self) -> ConcreteSignature {
            ConcreteSignature {
                inputs: vec![Type::Number],
                outputs: vec![],
            }
        }
    }
}
