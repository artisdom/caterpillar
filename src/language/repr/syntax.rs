use super::eval::value::ValueKind;

#[derive(Debug)]
pub struct SyntaxTree {
    pub elements: Vec<SyntaxElement>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum SyntaxElement {
    Block(SyntaxTree),

    /// A literal value
    ///
    /// This variant can represent `SyntaxElement`s that are not actually valid,
    /// as [`ValueKind`] can be a block, but a block is actually handled by a
    /// dedicated variant.
    ///
    /// Such an invalid `SyntaxElement` is never produced by the parser, and
    /// doing it like this makes the code handling `SyntaxElement`s simpler, and
    /// that's probably worth the small inconsistency.
    Literal(ValueKind),

    Word(String),
}
