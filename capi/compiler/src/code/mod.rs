#![allow(clippy::module_inception)]

pub mod search;

mod call_graph;
mod changes;
mod expression;
mod functions;
mod hash;
mod index;
mod location;
mod types;

pub use self::{
    call_graph::{CallGraph, Cluster},
    changes::{Changes, FunctionInUpdate, FunctionUpdate},
    expression::{Expression, UnresolvedCallToUserDefinedFunction},
    functions::{Branch, Function, NamedFunctions, Pattern},
    hash::Hash,
    index::{Index, IndexMap},
    location::{BranchLocation, FragmentLocation, FunctionLocation},
    types::{ConcreteSignature, Signature, Type, Types},
};
