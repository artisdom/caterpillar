use std::mem;

pub fn tokenize(source: String) -> Vec<Token> {
    let eager_tokens = vec![
        ("{", Token::FunctionOpen),
        ("}", Token::FunctionClose),
        ("|", Token::BranchHeadBoundary),
    ];

    let mut state = State::Initial;
    let mut buffer = Buffer::default();

    let mut tokens = Vec::new();

    for ch in source.chars() {
        match state {
            State::Initial => match ch {
                '#' => {
                    buffer.take_identifier(&mut tokens);
                    state = State::CommentStart;
                }
                ':' => {
                    tokens.push(Token::FunctionName {
                        name: buffer.take(),
                    });
                }
                ch if ch.is_whitespace() => {
                    buffer.take_identifier(&mut tokens);
                }
                ch => {
                    buffer.push(ch);

                    for (s, token) in &eager_tokens {
                        if buffer.inner.ends_with(s) {
                            buffer.inner.truncate(buffer.inner.len() - s.len());
                            buffer.take_identifier(&mut tokens);
                            tokens.push(token.clone());
                        }
                    }
                }
            },

            State::CommentStart | State::CommentText if ch == '\n' => {
                tokens.push(Token::Comment {
                    text: buffer.take(),
                });
                state = State::Initial;
            }
            State::CommentStart => match ch {
                ch if ch.is_whitespace() => {}
                ch => {
                    buffer.push(ch);
                    state = State::CommentText;
                }
            },
            State::CommentText => {
                buffer.push(ch);
                state = State::CommentText
            }
        }
    }

    tokens
}

#[derive(Clone, Debug)]
pub enum Token {
    Comment { text: String },

    BranchHeadBoundary,

    FunctionName { name: String },
    FunctionOpen,
    FunctionClose,

    Identifier { name: String },
}

enum State {
    Initial,
    CommentStart,
    CommentText,
}

#[derive(Default)]
struct Buffer {
    inner: String,
}

impl Buffer {
    pub fn push(&mut self, ch: char) {
        self.inner.push(ch);
    }

    pub fn take(&mut self) -> String {
        mem::take(&mut self.inner)
    }

    pub fn take_if_not_empty(&mut self) -> Option<String> {
        if self.inner.is_empty() {
            None
        } else {
            Some(self.take())
        }
    }

    pub fn take_identifier(&mut self, tokens: &mut Vec<Token>) {
        tokens.extend(
            self.take_if_not_empty()
                .map(|name| Token::Identifier { name }),
        );
    }
}
