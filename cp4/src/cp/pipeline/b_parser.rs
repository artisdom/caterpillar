use super::a_tokenizer::{Token, Tokenizer, TokenizerError};

pub struct Parser {
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self { tokenizer }
    }

    pub async fn next_token(
        &mut self,
    ) -> Result<Option<SyntaxElement>, ParserError> {
        self.parse().await
    }

    async fn parse(&mut self) -> Result<Option<SyntaxElement>, ParserError> {
        let token = self.tokenizer.next_token().await?;

        match token {
            Token::Ident(ident) => Ok(Some(SyntaxElement::Word(ident))),
            token => Err(ParserError::UnexpectedToken(token)),
        }
    }
}

pub enum SyntaxElement {
    Word(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error(transparent)]
    Tokenizer(#[from] TokenizerError),

    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
}

impl ParserError {
    pub fn is_no_more_chars(&self) -> bool {
        matches!(self, ParserError::Tokenizer(TokenizerError::NoMoreChars,),)
    }
}
