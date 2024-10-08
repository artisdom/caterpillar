//! # Types related to searching code
//!
//! Contains types that are returned by searches in code. Those types themselves
//! provide more convenient functionality for searching within them.

use std::ops::Deref;

use super::{
    Branch, BranchLocation, Fragment, FragmentLocation, Function,
    FunctionLocation,
};

/// # A function that was found by a search
pub struct FoundFunction {
    /// # The function that was found
    pub function: Function,

    /// # The location of the function that was found
    pub location: FunctionLocation,
}

impl FoundFunction {
    /// # Access the function's single branch
    ///
    /// Returns `None`, if the function does not have exactly one branch.
    pub fn find_single_branch(&self) -> Option<FoundBranch> {
        if self.branches.len() > 1 {
            return None;
        }

        self.branches
            .first_key_value()
            .map(|(&index, branch)| FoundBranch {
                branch,
                location: BranchLocation {
                    parent: Box::new(self.location.clone()),
                    index,
                },
            })
    }
}

impl Deref for FoundFunction {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

/// # A branch that was found by a search
pub struct FoundBranch<'r> {
    /// # The branch that was found
    pub branch: &'r Branch,

    /// # The location of the branch that was found
    pub location: BranchLocation,
}

impl FoundBranch<'_> {
    /// # Find the n-th fragment in the branch's body
    ///
    /// Returns `None`, if the branch has too few fragments.
    pub fn find_nth_fragment(&self, n: usize) -> Option<FoundFragment> {
        self.body
            .iter()
            .nth(n)
            .map(|(&index, fragment)| FoundFragment {
                fragment,
                location: FragmentLocation {
                    parent: Box::new(self.location.clone()),
                    index,
                },
            })
    }
}

impl Deref for FoundBranch<'_> {
    type Target = Branch;

    fn deref(&self) -> &Self::Target {
        self.branch
    }
}

/// # A fragment that was found by a search
pub struct FoundFragment<'r> {
    /// # The fragment that was found
    pub fragment: &'r Fragment,

    /// # The location of the fragment that was found
    pub location: FragmentLocation,
}

impl Deref for FoundFragment<'_> {
    type Target = Fragment;

    fn deref(&self) -> &Self::Target {
        self.fragment
    }
}
