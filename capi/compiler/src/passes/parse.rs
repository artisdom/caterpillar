use std::collections::VecDeque;

use crate::{
    fragments::Pattern,
    syntax::{Branch, Expression, Function},
};

use super::tokenize::Token;

pub fn parse(tokens: Vec<Token>) -> Vec<Function> {
    let mut tokens = Tokens {
        inner: tokens.into(),
    };
    let mut functions = Vec::new();

    while let Some(function) = parse_named_function(&mut tokens) {
        functions.push(function);
    }

    functions
}

fn parse_named_function(tokens: &mut Tokens) -> Option<Function> {
    let name = loop {
        if let Some(Token::Comment { .. }) = tokens.peek() {
            // Comments in the top-level context are currently ignored.
            tokens.take();
            continue;
        }

        match tokens.take()? {
            Token::FunctionName { name } => {
                break name;
            }
            token => {
                panic!("Unexpected token: {token:?}");
            }
        }
    };

    let mut function = parse_function(tokens)?;
    function.name = Some(name);

    Some(function)
}

fn parse_function(tokens: &mut Tokens) -> Option<Function> {
    let mut function = Function::default();

    match tokens.take()? {
        Token::FunctionStart => {}
        token => {
            panic!("Unexpected token: {token:?}");
        }
    }

    while let Some(branch) = parse_branch(tokens) {
        function.add_branch(branch);
    }

    match tokens.take()? {
        Token::FunctionEnd => {}
        token => {
            panic!("Unexpected token: {token:?}");
        }
    }

    Some(function)
}

fn parse_branch(tokens: &mut Tokens) -> Option<Branch> {
    match tokens.peek()? {
        Token::BranchStart => {
            tokens.take();
        }
        Token::FunctionEnd => {
            return None;
        }
        token => {
            panic!("Unexpected token: {token:?}");
        }
    }

    let mut parameters = Vec::new();
    let mut body = Vec::new();

    parse_branch_parameters(tokens, &mut parameters);
    parse_branch_body(tokens, &mut body)?;

    Some(Branch { parameters, body })
}

fn parse_branch_parameters(tokens: &mut Tokens, parameters: &mut Vec<Pattern>) {
    while let Some(token) = tokens.take() {
        match token {
            Token::Identifier { name } => {
                parameters.push(Pattern::Identifier { name });
            }
            Token::IntegerLiteral { value } => {
                parameters.push(Pattern::Literal {
                    value: value.into(),
                });
            }
            Token::BranchBodyStart => {
                break;
            }
            token => {
                panic!("Unexpected token: {token:?}");
            }
        }
    }
}

fn parse_branch_body(
    tokens: &mut Tokens,
    body: &mut Vec<Expression>,
) -> Option<()> {
    while let Some(token) = tokens.peek() {
        match token {
            Token::FunctionStart => {
                body.extend(
                    parse_function(tokens)
                        .map(|function| Expression::Function { function }),
                );
            }
            Token::BranchStart | Token::FunctionEnd => {
                break;
            }
            _ => match tokens.take()? {
                Token::Comment { text } => {
                    body.push(Expression::Comment { text });
                }
                Token::Identifier { name } => {
                    body.push(Expression::UnresolvedIdentifier {
                        name,
                        is_known_to_be_in_tail_position: false,
                        is_known_to_be_call_to_user_defined_function: None,
                    });
                }
                Token::IntegerLiteral { value } => {
                    body.push(Expression::Value(value.into()));
                }
                token => {
                    panic!("Unexpected token: {token:?}");
                }
            },
        }
    }

    Some(())
}

struct Tokens {
    inner: VecDeque<Token>,
}

impl Tokens {
    pub fn peek(&self) -> Option<&Token> {
        self.inner.front()
    }

    pub fn take(&mut self) -> Option<Token> {
        self.inner.pop_front()
    }
}
