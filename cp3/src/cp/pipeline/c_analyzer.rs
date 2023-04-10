use crate::cp::{
    expressions::{Expression, ExpressionGraph},
    functions::Function,
    syntax::{SyntaxElement, SyntaxTree},
    Functions,
};

pub fn analyze(
    module: &str,
    syntax_tree: SyntaxTree,
    functions: &mut Functions,
) -> ExpressionGraph {
    let mut expressions = Vec::new();

    for syntax_element in syntax_tree {
        let expression = match syntax_element {
            SyntaxElement::Module { name, body } => {
                expressions.extend(analyze(&name, body, functions));
                continue;
            }
            SyntaxElement::Function { name, body } => {
                let body = analyze(module, body, functions);
                let function = Function {
                    body,
                    module: module.into(),
                };

                functions.define_function(name, function);

                continue;
            }
            SyntaxElement::Test { name, body } => {
                let body = analyze(module, body, functions);
                let function = Function {
                    body,
                    module: module.into(),
                };

                functions.define_test(name, function);

                continue;
            }
            SyntaxElement::Binding(binding) => Expression::Binding(binding),
            SyntaxElement::Array { syntax_tree } => {
                let expressions = analyze(module, syntax_tree, functions);
                Expression::Array {
                    syntax_tree: expressions,
                }
            }
            SyntaxElement::Block { syntax_tree } => {
                let expressions = analyze(module, syntax_tree, functions);
                Expression::Block {
                    syntax_tree: expressions,
                }
            }
            SyntaxElement::String(string) => Expression::String(string),
            SyntaxElement::Word(word) => Expression::Word(word),
        };

        expressions.push(expression);
    }

    ExpressionGraph::from(expressions)
}
