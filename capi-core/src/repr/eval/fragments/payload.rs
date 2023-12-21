use crate::repr::eval::value::ValuePayload;

use super::FragmentId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FragmentPayload {
    Array {
        start: FragmentId,
    },
    Value(ValuePayload),
    Word(String),

    /// Terminates a context
    ///
    /// By convention, fragments within a block use the fragment *after* the
    /// block as their parents. This is done for practical reasons, as the ID
    /// of the next fragment is available when a parent ID is needed, while the
    /// ID of the block itself or the fragment before it isn't. And they can't
    /// be made available either, as they depend on the block contents, which
    /// would result in a circular dependency.
    ///
    /// However, this means that blocks *must not* be the last fragment in a
    /// context, or the items within such blocks are no longer uniquely
    /// addressable.
    ///
    /// This is why terminators exist. They terminate every context, and thus
    /// make sure that a unique parent is provided for the fragments in any
    /// block.
    Terminator,
}

impl FragmentPayload {
    pub fn display_short(&self) -> String {
        match self {
            Self::Array { start } => {
                let start = start.display_short();
                format!("array [ {start} ]")
            }
            Self::Value(value) => {
                let value = value.display_short();
                format!("value `{value}`")
            }
            Self::Word(word) => format!("word `{word}`"),
            Self::Terminator => "terminator".to_string(),
        }
    }

    pub(super) fn hash(&self, hasher: &mut blake3::Hasher) {
        match self {
            Self::Array { start } => {
                hasher.update(b"array");
                hasher.update(start.hash.as_bytes());
            }
            Self::Value(value) => {
                hasher.update(b"value");
                value.hash(hasher);
            }
            Self::Word(word) => {
                hasher.update(b"word");
                hasher.update(word.as_bytes());
            }
            Self::Terminator => {
                hasher.update(b"terminator");
            }
        }
    }
}
