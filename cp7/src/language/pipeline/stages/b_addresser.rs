use std::collections::HashMap;

use crate::language::pipeline::concepts::tokens::{
    AddressedToken, Token, TokenAddress, Tokens,
};

pub fn address(tokens: impl IntoIterator<Item = Token>) -> Tokens {
    let mut left_to_right = HashMap::new();
    let mut right_to_left = HashMap::new();

    let left =
        address_token(None, tokens, &mut left_to_right, &mut right_to_left);

    Tokens {
        left,
        left_to_right,
        right_to_left,
    }
}

fn address_token(
    left: Option<TokenAddress>,
    tokens: impl IntoIterator<Item = Token>,
    left_to_right: &mut HashMap<TokenAddress, AddressedToken>,
    right_to_left: &mut HashMap<TokenAddress, AddressedToken>,
) -> Option<TokenAddress> {
    let mut tokens = tokens.into_iter();
    let token = tokens.next()?;

    let address_left = build_address(&token, left);
    let right =
        address_token(Some(address_left), tokens, left_to_right, right_to_left);
    let address_right = build_address(&token, right);

    let addressed_token = AddressedToken { token, left, right };

    left_to_right.insert(address_left, addressed_token.clone());
    right_to_left.insert(address_right, addressed_token);

    Some(address_right)
}

fn build_address(
    token: &Token,
    neighbor: Option<TokenAddress>,
) -> TokenAddress {
    let mut hasher = blake3::Hasher::new();

    hasher.update(token.to_string().as_bytes());
    if let Some(neighbor) = neighbor {
        hasher.update(neighbor.0.as_bytes());
    }

    let hash = hasher.finalize();
    TokenAddress(hash)
}
