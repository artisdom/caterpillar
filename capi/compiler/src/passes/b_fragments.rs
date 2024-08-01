use std::collections::BTreeMap;

use crate::repr::{
    fragments::{
        Fragment, FragmentExpression, FragmentId, FragmentMap, FragmentParent,
        FragmentPayload, Fragments, Function,
    },
    syntax::{Expression, IdentifierTarget, Script},
};

pub fn generate_fragments(script: Script) -> Fragments {
    let mut fragments = FragmentMap {
        inner: BTreeMap::new(),
    };
    let mut by_function = Vec::new();

    for function in script.functions {
        let start = compile_block(
            function.body,
            FragmentParent::Function {
                name: function.name.clone(),
            },
            &mut fragments,
        );

        by_function.push(Function {
            name: function.name,
            args: function.args,
            start,
        });
    }

    Fragments {
        inner: fragments,
        by_function,
    }
}

fn compile_block(
    expressions: Vec<Expression>,
    parent: FragmentParent,
    fragments: &mut FragmentMap,
) -> FragmentId {
    let mut next = {
        let terminator = Fragment {
            parent: Some(parent.clone()),
            payload: FragmentPayload::Terminator,
        };
        let terminator_id = terminator.id();

        fragments.inner.insert(terminator_id, terminator);

        terminator_id
    };

    for expression in expressions.into_iter().rev() {
        let fragment =
            compile_expression(expression, parent.clone(), next, fragments);

        next = fragment.id();

        fragments.inner.insert(fragment.id(), fragment);
    }

    next
}

fn compile_expression(
    expression: Expression,
    parent: FragmentParent,
    next: FragmentId,
    fragments: &mut FragmentMap,
) -> Fragment {
    let expression = match expression {
        Expression::Binding { names } => {
            FragmentExpression::BindingDefinitions { names }
        }
        Expression::Block { body, environment } => {
            let start = compile_block(
                body,
                FragmentParent::Fragment { id: next },
                fragments,
            );
            FragmentExpression::Block { start, environment }
        }
        Expression::Comment { text } => FragmentExpression::Comment { text },
        Expression::Identifier { name, target } => match target {
            Some(IdentifierTarget::Binding) => {
                FragmentExpression::ResolvedBinding { name }
            }
            Some(IdentifierTarget::BuiltinFunction) => {
                FragmentExpression::ResolvedBuiltinFunction { name }
            }
            Some(IdentifierTarget::HostFunction) => {
                FragmentExpression::ResolvedHostFunction { name }
            }
            Some(IdentifierTarget::UserFunction) => {
                FragmentExpression::ResolvedUserFunction { name }
            }
            Some(IdentifierTarget::SelfRecursion) => {
                let next_fragment = fragments.inner.get(&next).expect(
                    "The next fragment should have been generated before this \
                    one. If it doesn't reference anything, that's a bug in \
                    this compiler pass.",
                );

                if let FragmentPayload::Terminator = next_fragment.payload {
                    FragmentExpression::TailRecursiveCall { name }
                } else {
                    FragmentExpression::ResolvedUserFunction { name }
                }
            }
            None => FragmentExpression::UnresolvedIdentifier { name },
        },
        Expression::Value(value) => FragmentExpression::Value(value),
    };

    Fragment {
        parent: Some(parent),
        payload: FragmentPayload::Expression { expression, next },
    }
}

#[cfg(test)]
mod tests {
    use capi_process::Value;

    use crate::{
        passes::generate_fragments,
        repr::{
            fragments::{
                Fragment, FragmentExpression, FragmentParent, FragmentPayload,
                Fragments,
            },
            syntax::Script,
        },
    };

    #[test]
    fn duplicate_payload() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.v(1).v(1);
        });

        let fragments = generate_fragments(script);

        let body = body(fragments);
        assert_eq!(
            body,
            [
                FragmentExpression::Value(Value(1i32.to_le_bytes())),
                FragmentExpression::Value(Value(1i32.to_le_bytes())),
            ]
        );
    }

    #[test]
    fn terminator() {
        let mut script = Script::default();
        script.function("f", [], |_| {});

        let mut fragments = generate_fragments(script);

        let start = fragments.by_function.remove(0).start;
        let last_fragment = fragments.inner.iter_from(start).last().unwrap();
        assert_eq!(last_fragment.payload, FragmentPayload::Terminator);
    }

    #[test]
    fn block_parent() {
        let mut script = Script::default();
        script.function("f", [], |s| {
            s.block(|_| {});
        });

        let mut fragments = generate_fragments(script);

        let start = fragments.by_function.remove(0).start;
        let function_fragments =
            fragments.inner.iter_from(start).collect::<Vec<_>>();
        let block_fragments = {
            let Fragment {
                payload:
                    FragmentPayload::Expression {
                        expression: FragmentExpression::Block { start, .. },
                        ..
                    },
                ..
            } = function_fragments[0]
            else {
                panic!("Expected block")
            };

            fragments.inner.iter_from(*start).collect::<Vec<_>>()
        };

        assert_eq!(
            function_fragments[0].parent,
            Some(FragmentParent::Function {
                name: String::from("f")
            }),
        );
        assert_eq!(
            block_fragments[0].parent,
            Some(FragmentParent::Fragment {
                id: function_fragments[1].id()
            }),
        );
    }

    fn body(mut fragments: Fragments) -> Vec<FragmentExpression> {
        let mut body = Vec::new();

        let start = fragments.by_function.remove(0).start;

        body.extend(fragments.inner.iter_from(start).filter_map(|fragment| {
            match &fragment.payload {
                FragmentPayload::Expression { expression, .. } => {
                    Some(expression.clone())
                }
                FragmentPayload::Function { .. } => {
                    unreachable!(
                        "This test suite does not define functions within \
                        function bodies."
                    );
                }
                FragmentPayload::Terminator => None,
            }
        }));

        body
    }
}
