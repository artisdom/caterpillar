use std::collections::VecDeque;

use enum_variant_type::EnumVariantType;

pub fn tokenize(code: &str) -> Tokens {
    let mut tokens = VecDeque::new();

    for token in code.split_whitespace() {
        let token = tokenize_token(token);
        tokens.push_back(token);
    }

    Tokens::from(tokens)
}

fn tokenize_token(token: &str) -> Token {
    match token {
        "{" => return Token::CurlyBracketOpen,
        "}" => return Token::CurlyBracketClose,
        _ => {}
    }

    if let Some(("", symbol)) = token.split_once(':') {
        return Token::Symbol(symbol.into());
    }

    if let Ok(number) = token.parse::<i64>() {
        return Token::Number(number);
    }

    Token::FnRef(token.into())
}

pub struct Tokens {
    inner: VecDeque<Token>,
}

impl Tokens {
    pub fn peek(&self) -> Result<Token, NoMoreTokens> {
        self.inner.front().cloned().ok_or(NoMoreTokens)
    }

    pub fn next(&mut self) -> Result<Token, NoMoreTokens> {
        self.inner.pop_front().ok_or(NoMoreTokens)
    }
}

impl From<VecDeque<Token>> for Tokens {
    fn from(tokens: VecDeque<Token>) -> Self {
        Self { inner: tokens }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, EnumVariantType)]
#[evt(module = "token")]
pub enum Token {
    CurlyBracketOpen,
    CurlyBracketClose,
    FnRef(String),
    Number(i64),
    Symbol(String),
}

#[derive(Debug, thiserror::Error)]
#[error("No more tokens")]
pub struct NoMoreTokens;
