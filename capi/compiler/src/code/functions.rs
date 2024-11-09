use std::collections::BTreeSet;

use capi_runtime::Value;

use crate::code::Index;

use super::{
    BranchLocation, Cluster, Expression, ExpressionLocation, FunctionLocation,
    Hash, IndexMap, Located,
};

/// # All functions in a program
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Functions {
    /// # The named functions
    ///
    /// At this point, all named functions live in a single root context, and
    /// are addressed by an index into that root context. The language is
    /// expected to grow a module system in the future, and then this will
    /// change.
    pub named: IndexMap<NamedFunction>,
}

impl Functions {
    /// # Find the named function with the provided hash
    pub fn find_named_by_hash(
        &self,
        hash: &Hash<Function>,
    ) -> Option<Located<&NamedFunction>> {
        self.named.iter().find_map(|(&index, function)| {
            if &Hash::new(&function.inner) == hash {
                Some(Located {
                    fragment: function,
                    location: index,
                })
            } else {
                None
            }
        })
    }

    /// # Find the named function with the provided index
    pub fn find_named_by_index(
        &self,
        index: &Index<NamedFunction>,
    ) -> Option<Located<&NamedFunction>> {
        let function = self.named.get(index)?;
        Some(Located {
            fragment: function,
            location: *index,
        })
    }

    /// # Find the function with the provided name
    pub fn find_by_name(&self, name: &str) -> Option<Located<&NamedFunction>> {
        self.named.iter().find_map(|(&index, function)| {
            if function.name == name {
                Some(Located {
                    fragment: function,
                    location: index,
                })
            } else {
                None
            }
        })
    }

    /// # Find the branch at the given location
    pub fn find_branch_by_location(
        &self,
        location: &BranchLocation,
    ) -> Option<&Branch> {
        let function = self.find_function_by_location(&location.parent)?;
        function.branches.get(&location.index)
    }

    /// # Find the expression at the given location
    pub fn find_expression_by_location(
        &self,
        location: &ExpressionLocation,
    ) -> Option<&Expression> {
        let branch = self.find_branch_by_location(&location.parent)?;
        branch.body.get(&location.index)
    }

    /// # Find the function at the given location
    ///
    /// This includes both named and anonymous functions.
    pub fn find_function_by_location(
        &self,
        location: &FunctionLocation,
    ) -> Option<&Function> {
        match location {
            FunctionLocation::NamedFunction { index } => {
                self.named.get(index).map(|function| &function.inner)
            }
            FunctionLocation::AnonymousFunction { location } => {
                let expression = self.find_expression_by_location(location)?;
                expression.as_function()
            }
        }
    }

    /// # Iterate over the named functions
    pub fn named_functions(
        &self,
    ) -> impl Iterator<Item = Located<&NamedFunction>> {
        self.named.iter().map(|(index, function)| Located {
            fragment: function,
            location: *index,
        })
    }

    /// # Iterate over the named functions mutably
    pub fn named_functions_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut NamedFunction> {
        self.named.values_mut()
    }

    /// # Consume this instance and return an iterator over the named functions
    pub fn into_named_functions(self) -> impl Iterator<Item = NamedFunction> {
        self.named.into_values()
    }
}

/// # A function that has a name
///
/// Named functions are defined in the top-level context. Functions that do not
/// have a name are anonymous, and are defined as literal values within other
/// functions.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct NamedFunction {
    /// # The name of the function
    pub name: String,

    /// # The function
    pub inner: Function,
}

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Function {
    /// # The branches of this function
    ///
    /// A function is made up of one or more branches. When a function is
    /// called, its arguments are matched against the parameters of each branch,
    /// until one branch matches. This branch is then evaluated.
    pub branches: IndexMap<Branch>,

    /// # Values captured by the function from a parent scope
    ///
    /// All functions in Caterpillar are closures that can use values from
    /// parent scopes. The names of those values are stored here.
    pub environment: BTreeSet<String>,

    /// # The index of this function within its cluster
    ///
    /// This is defined for named functions only. The value is `None` for
    /// anonymous functions.
    pub index_in_cluster: Option<Index<(Function, Cluster)>>,
}

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Branch {
    pub parameters: Vec<Pattern>,

    /// # The body of the branch
    pub body: IndexMap<Expression>,
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub enum Pattern {
    Identifier { name: String },
    Literal { value: Value },
}
