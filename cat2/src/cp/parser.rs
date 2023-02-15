use super::Token;

pub type Expressions = Vec<Expression>;

pub enum Expression {
    Fn(String),
}

pub fn parse(tokens: impl IntoIterator<Item = Token>) -> Expressions {
    let mut tokens = tokens.into_iter();
    let mut expressions = Vec::new();

    while let Some(expression) = parse_expression(&mut tokens, None) {
        expressions.push(expression);
    }

    expressions
}

fn parse_expression(
    mut tokens: impl Iterator<Item = Token>,
    terminator: Option<&Token>,
) -> Option<Expression> {
    let token = tokens.next()?;

    if Some(&token) == terminator {
        return None;
    }

    match token {
        Token::Fn(name) => Some(Expression::Fn(name)),
        Token::BlockOpen => {
            todo!("`{{` not supported yet")
        }
        Token::BlockClose => {
            todo!("`}}` not supported yet")
        }
    }
}
