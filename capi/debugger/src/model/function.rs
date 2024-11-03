use anyhow::anyhow;
use capi_compiler::{
    code::{
        self, BranchLocation, Cluster, FragmentLocation, FunctionLocation,
        NamedFunctions, Types,
    },
    source_map::SourceMap,
};
use capi_runtime::Effect;

use super::{Breakpoints, DebugBranch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugFunction {
    pub name: Option<String>,
    pub branches: Vec<DebugBranch>,
}

impl DebugFunction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        function: code::Function,
        location: FunctionLocation,
        active_fragment: Option<&FragmentLocation>,
        is_innermost_active_function: bool,
        cluster: &Cluster,
        named_functions: &NamedFunctions,
        types: &Types,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effect: Option<&Effect>,
    ) -> Self {
        let name = function.name;
        let branches = function
            .branches
            .into_iter()
            .map(|(index, branch)| {
                DebugBranch::new(
                    branch,
                    BranchLocation {
                        parent: Box::new(location.clone()),
                        index,
                    },
                    active_fragment,
                    is_innermost_active_function,
                    cluster,
                    named_functions,
                    types,
                    source_map,
                    breakpoints,
                    effect,
                )
            })
            .collect();

        Self { name, branches }
    }

    pub fn active_branch(&self) -> anyhow::Result<&DebugBranch> {
        self.branches
            .iter()
            .find(|branch| branch.is_active)
            .ok_or_else(|| {
                anyhow!(
                    "Expected to find active branch in function, but none is."
                )
            })
    }
}
