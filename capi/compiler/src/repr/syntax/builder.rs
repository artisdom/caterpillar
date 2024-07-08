use capi_process::Value;

use crate::repr::syntax::{Expression, ExpressionKind};

#[derive(Debug)]
pub struct SyntaxBuilder<'r> {
    expressions: &'r mut Vec<Expression>,
}

impl<'r> SyntaxBuilder<'r> {
    pub fn new(expressions: &'r mut Vec<Expression>) -> Self {
        Self { expressions }
    }

    pub fn bind(
        &mut self,
        names: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.push_expression(ExpressionKind::Binding {
            names: names.into_iter().map(Into::into).collect(),
        })
    }

    pub fn c(&mut self, text: &str) -> &mut Self {
        self.push_expression(ExpressionKind::Comment { text: text.into() })
    }

    pub fn v(&mut self, value: impl Into<Value>) -> &mut Self {
        self.push_expression(ExpressionKind::Value(value.into()))
    }

    pub fn w(&mut self, name: &str) -> &mut Self {
        self.push_expression(ExpressionKind::Word { name: name.into() })
    }

    fn push_expression(&mut self, kind: ExpressionKind) -> &mut Self {
        self.expressions.push(Expression::new(kind));
        self
    }
}
