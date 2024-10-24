use capi_runtime::Value;

use crate::{hash::Hash, intrinsics::IntrinsicFunction};

use super::{Cluster, Function, Index};

/// # A pre-compiled piece of code
///
/// Fragments are the core of Caterpillar's code representation, the smallest
/// units of code.
///
/// They are the result of a partial compilation process. This is called
/// pre-compilation, because it happens before the actual translation into
/// instructions that the runtime can interpret.
///
///
/// ## Error Handling
///
/// An important feature of this code representation is, that it can be the
/// result of a failed compilation process. If, for example, an identifier can't
/// be resolved, this is still encoded as a fragment.
///
/// As a result, other code that is not affected can still be executed (as part
/// of automated testing, for example). But also, the rich representation
/// produced by the pre-compilation process is still available for display by
/// tooling, regardless of any isolated errors.
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
pub enum Fragment {
    /// # A reference to a local binding
    Binding {
        /// # The name of the binding
        name: String,

        /// # The index of the binding
        ///
        /// The index is derived from the index of the binding in the parameter
        /// list of its branch. Non-identifiers are ignored in this.
        ///
        /// The index determines the position within the local stack frame,
        /// where the binding is stored.
        ///
        /// ## Implementation Note
        ///
        /// As of this writing, bindings are not actually stored like described
        /// here. This is a work in progress.
        index: u32,
    },

    /// # A call to a function defined by the host
    ///
    /// Host functions present as functions to the user. But contrary to regular
    /// functions, they have no representation in the form of Caterpillar code.
    ///
    /// The compiler translates calls to host functions into instructions that
    /// trigger a specific effect. This effect is then handled by the host in
    /// whatever way it deems appropriate.
    CallToHostFunction {
        /// # A number that identifies the specific effect
        ///
        /// The meaning of this number is only known to the host. The compiler
        /// doesn't know, nor doesn't need to know, what it means.
        effect_number: u8,
    },

    /// # A call to a compiler-intrinsic function
    ///
    /// Intrinsic functions are implemented in the compiler. Calls to them are
    /// directly translated into a series of instructions, which provide the
    /// desired behavior.
    CallToIntrinsicFunction {
        /// # The intrinsic function being called
        intrinsic: IntrinsicFunction,

        /// # Indicate whether the call is in tail position
        ///
        /// This is relevant, as intrinsics can trigger calls to user-defined
        /// functions, which might necessitate tail call elimination.
        is_tail_call: bool,
    },

    /// # A call to a user-defined function
    CallToUserDefinedFunction {
        /// # The hash of the function being called
        hash: Hash<Function>,

        /// # Indicate whether the call is in tail position
        ///
        /// This is relevant as function calls might necessitate tail call
        /// elimination.
        is_tail_call: bool,
    },

    /// # A recursive call to a user-defined function
    ///
    /// This call can either be directly recursive (a function is calling
    /// itself), or mutually recursive (the function is calling another function
    /// that directly or indirectly calls the original function).
    ///
    /// This needs to be handled separately from non-recursive calls, as those
    /// non-recursive calls reference the callee by hash. In a recursive call,
    /// this is not possible. It would result in a circular dependency when
    /// creating the hash of the callee, since that would depend on the hash of
    /// caller, which would depend on the hash of the callee again.
    CallToUserDefinedFunctionRecursive {
        /// # The index of the called function within its cluster
        ///
        /// During compilation, functions are grouped into clusters. A cluster
        /// either contains a single functions, or a group of mutually recursive
        /// function. All mutually recursive functions are part of a single
        /// cluster.
        ///
        /// If this is a function calling itself, the index is always `0`. If
        /// the calling function is part of a cluster of mutually recursive
        /// functions, the index identifies the called function within the
        /// cluster.
        index: Index<(Function, Cluster)>,

        /// # Indicate whether the call is in tail position
        ///
        /// This is relevant as function calls might necessitate tail call
        /// elimination.
        is_tail_call: bool,
    },

    /// # A comment, which does not influence the execution of the code
    Comment {
        /// # The text of the comment
        text: String,
    },

    /// # A function literal
    ///
    /// This is used to represent both anonymous functions that are used where
    /// an expression is accepted, as well as named functions defined in the
    /// top-level context.
    Function {
        /// # The function defined by this literal
        function: Function,
    },

    /// # An unresolved identifier
    ///
    /// This is the result of a compiler error.
    UnresolvedIdentifier {
        /// # The name of the unresolved identifier
        name: String,

        /// # Indicate whether the identifier is known to be in tail position
        ///
        /// An expression is in tail position, if it is the last expression in
        /// its function or block.
        ///
        /// This starts out being `false` for all expressions, and will
        /// eventually be filled in by a dedicated compiler pass.
        ///
        /// This flag is relevant for tail call elimination. It is only needed
        /// for identifiers, because only identifiers can result in tail calls.
        is_known_to_be_in_tail_position: bool,

        /// # Indicate whether the identifier is known to be a function call
        ///
        /// This starts out as `false` and might later get updated by the
        /// respective compiler pass.
        is_known_to_be_call_to_user_defined_function:
            Option<UnresolvedCallToUserDefinedFunction>,
    },

    /// # A literal value
    Value(Value),
}

impl Fragment {
    pub fn as_call_to_function(&self) -> Option<&Hash<Function>> {
        let Fragment::CallToUserDefinedFunction { hash, .. } = self else {
            return None;
        };

        Some(hash)
    }

    pub fn as_comment(&self) -> Option<&String> {
        let Fragment::Comment { text } = self else {
            return None;
        };

        Some(text)
    }

    /// # Convert the fragment to a `Function`
    ///
    /// Return `None`, if this is a different kind of fragment.
    pub fn as_function(&self) -> Option<&Function> {
        let Fragment::Function { function } = self else {
            return None;
        };

        Some(function)
    }
}

/// # The information that is currently known about an unresolved function call
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
pub struct UnresolvedCallToUserDefinedFunction {
    /// # Indicate whether the call is known to be recursive
    ///
    /// Starts out as `None`, until it might get filled in by the respective
    /// compiler pass. In that case, the index of the function within the
    /// cluster is provided, which is later needed to resolve the call.
    pub is_known_to_be_recursive_call: Option<Index<(Function, Cluster)>>,
}
