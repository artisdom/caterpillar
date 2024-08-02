use std::collections::BTreeSet;

use capi_process::Value;

use crate::repr::syntax::Expression;

#[derive(Debug)]
pub struct SyntaxBuilder<'r> {
    expressions: &'r mut Vec<Expression>,
}

impl<'r> SyntaxBuilder<'r> {
    pub fn new(expressions: &'r mut Vec<Expression>) -> Self {
        Self { expressions }
    }

    pub fn block(&mut self, f: impl FnOnce(&mut SyntaxBuilder)) -> &mut Self {
        let mut body = Vec::new();
        f(&mut SyntaxBuilder::new(&mut body));

        self.push_expression(Expression::Block {
            body,
            environment: BTreeSet::new(),
        })
    }

    pub fn bind(
        &mut self,
        names: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.push_expression(Expression::Binding {
            names: names.into_iter().map(Into::into).collect(),
        })
    }

    pub fn c(&mut self, text: &str) -> &mut Self {
        self.push_expression(Expression::Comment { text: text.into() })
    }

    pub fn ident(&mut self, name: &str) -> &mut Self {
        self.push_expression(Expression::Identifier {
            name: name.into(),
            target: None,
            is_known_to_be_in_tail_position: false,
        })
    }

    pub fn v(&mut self, value: impl Into<Value>) -> &mut Self {
        self.push_expression(Expression::Value(value.into()))
    }

    fn push_expression(&mut self, expression: Expression) -> &mut Self {
        self.expressions.push(expression);
        self
    }
}
