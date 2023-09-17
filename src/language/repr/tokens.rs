use enum_variant_type::EnumVariantType;

use super::eval::value::ValueKind;

#[derive(Clone, Debug, Eq, PartialEq, EnumVariantType)]
#[evt(module = "token")]
pub enum Token {
    CurlyBracketOpen,
    CurlyBracketClose,

    /// A literal value
    ///
    /// This variant can represent `Token`s that are not actually valid, as
    /// [`ValueKind`] can be a block, but blocks don't exist on the token level.
    ///
    /// Such an invalid `Token` is never produced by the tokenizer, and doing it
    /// like this makes the code handling `Token`s simpler, and that's probably
    /// worth the small inconsistency.
    Literal(ValueKind),

    Word(String),
}
