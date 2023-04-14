use std::mem;

use map_macro::map;

use crate::cp::{
    keywords::Keyword,
    tokens::{Token, STRING_DELIMITER},
};

#[derive(Debug)]
pub enum Tokenizer {
    Searching,
    ProcessingAny { buf: String },
    ProcessingString { buf: String },
}

pub fn tokenizer() -> Tokenizer {
    Tokenizer::Searching
}

pub fn push_char(ch: char, tokenizer: Tokenizer) -> (Tokenizer, Vec<Token>) {
    match tokenizer {
        Tokenizer::Searching => {
            if ch.is_whitespace() {
                return (tokenizer, vec![]);
            }

            if ch == STRING_DELIMITER {
                return (
                    Tokenizer::ProcessingString { buf: String::new() },
                    vec![],
                );
            }

            let buf = String::from(ch);
            (Tokenizer::ProcessingAny { buf }, vec![])
        }
        Tokenizer::ProcessingAny { mut buf } => {
            let mut tokens = match_eagerly(&buf);
            if !tokens.is_empty() {
                buf.clear();

                if ch.is_whitespace() {
                    return (Tokenizer::Searching, tokens);
                }
            }

            if ch == STRING_DELIMITER {
                tokens.extend(match_delimited(&buf));
                return (
                    Tokenizer::ProcessingString { buf: String::new() },
                    tokens,
                );
            }

            if !ch.is_whitespace() {
                buf.push(ch);
                return (Tokenizer::ProcessingAny { buf }, tokens);
            }

            if !buf.is_empty() {
                tokens.extend(match_delimited(&buf));
                return (Tokenizer::Searching, tokens);
            }

            (Tokenizer::ProcessingAny { buf }, tokens)
        }
        Tokenizer::ProcessingString { mut buf } => {
            if ch == STRING_DELIMITER {
                let string = mem::take(&mut buf);
                let token = Token::String(string);

                return (Tokenizer::Searching, vec![token]);
            }

            buf.push(ch);

            (Tokenizer::ProcessingString { buf }, vec![])
        }
    }
}

pub fn finalize(tokenizer: Tokenizer) -> Vec<Token> {
    let mut tokens = Vec::new();

    if let Tokenizer::ProcessingAny { buf } = tokenizer {
        tokens.extend(match_delimited(&buf));
    }

    tokens
}

fn match_eagerly(buf: &str) -> Vec<Token> {
    let mut delimiters = map! {
        "=>" => Token::BindingOperator,
        "." => Token::Period,
        "{" => Token::CurlyBracketOpen,
        "}" => Token::CurlyBracketClose,
        "(" => Token::RoundBracketOpen,
        ")" => Token::RoundBracketClose,
        "[" => Token::SquareBracketOpen,
        "]" => Token::SquareBracketClose,
    };

    if let Some(token) = delimiters.remove(buf) {
        return vec![token];
    }

    for (delimiter, token) in delimiters {
        if let Some((first_token, "")) = buf.split_once(delimiter) {
            let mut tokens = Vec::new();

            tokens.extend(match_delimited(first_token));
            tokens.push(token);

            return tokens;
        }
    }

    vec![]
}

fn match_delimited(buf: &str) -> Option<Token> {
    // This function is only ever called with a `buf` that has already been
    // matched against delimiters before. We don't need to check again, if any
    // delimiters are in here.

    if let Some(keyword) = Keyword::parse(buf) {
        return Some(Token::Keyword(keyword));
    }
    if let Some(("", symbol)) = buf.split_once(':') {
        return Some(Token::Symbol(symbol.into()));
    }

    Some(Token::Ident(buf.into()))
}
