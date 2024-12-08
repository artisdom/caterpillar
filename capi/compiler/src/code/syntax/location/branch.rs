use std::{fmt, iter};

use crate::code::{
    syntax::{
        Branch, Expression, Function, IdentifierIndex, Member, Pattern,
        SyntaxSignature, SyntaxTree,
    },
    Index,
};

use super::{located::HasLocation, FunctionLocation, Located, MemberLocation};

impl HasLocation for Branch {
    type Location = BranchLocation;
}

impl<'r> Located<&'r Branch> {
    /// # Iterate over the members of the branch's body
    pub fn body(self) -> impl DoubleEndedIterator<Item = Located<&'r Member>> {
        let location = self.location.clone();

        self.body.iter().map(move |(&index, member)| Located {
            fragment: member,
            location: MemberLocation {
                parent: Box::new(location.clone()),
                index,
            },
        })
    }

    /// # Iterate over the type-annotated expressions in the branch's body
    pub fn annotated_expressions(
        self,
    ) -> impl DoubleEndedIterator<
        Item = (Located<&'r Expression>, Option<&'r SyntaxSignature>),
    > {
        self.body().filter_map(|member| member.into_expression())
    }

    /// # Iterate over the expressions in the branch's body
    pub fn expressions(
        self,
    ) -> impl DoubleEndedIterator<Item = Located<&'r Expression>> {
        self.annotated_expressions()
            .map(|(expression, _)| expression)
    }

    /// # Iterate over all local functions in this branch, recursively
    pub fn all_local_functions(
        self,
    ) -> impl Iterator<Item = Located<&'r Function>> {
        self.expressions()
            .filter_map(|expression| expression.into_local_function())
            .flat_map(|function| {
                iter::once(function.clone())
                    .chain(function.all_local_functions())
            })
    }

    /// # Compute the index of the identifier with the given name, if any
    pub fn identifier_index(&self, name: &str) -> Option<IdentifierIndex> {
        let indices = iter::successors(Some(0), |i| Some(i + 1));
        let identifiers =
            self.parameters.iter().filter_map(|pattern| match pattern {
                Pattern::Identifier { name } => Some(name),
                Pattern::Literal { .. } => None,
            });

        indices.zip(identifiers).find_map(|(i, identifier)| {
            (identifier == name).then_some(IdentifierIndex {
                value: i,
                branch: self.location.clone(),
            })
        })
    }
}

/// # The location of a branch in the source code
#[derive(
    Clone,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct BranchLocation {
    pub parent: Box<FunctionLocation>,
    pub index: Index<Branch>,
}

impl BranchLocation {
    /// # Create a helper that implements [`fmt::Display`]
    pub fn display<'r>(
        &'r self,
        syntax_tree: &'r SyntaxTree,
    ) -> BranchLocationDisplay<'r> {
        BranchLocationDisplay {
            location: self,
            syntax_tree,
        }
    }
}

/// # Helper struct to display [`BranchLocation`]
///
/// Implements [`fmt::Display`], which [`BranchLocation`] itself doesn't.
pub struct BranchLocationDisplay<'r> {
    location: &'r BranchLocation,
    syntax_tree: &'r SyntaxTree,
}

impl fmt::Display for BranchLocationDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "branch {} of {}",
            self.location.index,
            self.location.parent.display(self.syntax_tree),
        )
    }
}
