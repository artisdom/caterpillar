use capi_compiler::{
    fragments::{self, FragmentId, Fragments},
    source_map::SourceMap,
};
use capi_process::{Breakpoints, Effect};

use super::DebugBranch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugFunction {
    pub name: Option<String>,
    pub branches: Vec<DebugBranch>,
}

impl DebugFunction {
    pub fn new(
        function: fragments::Function,
        active_fragment: Option<FragmentId>,
        fragments: &Fragments,
        source_map: &SourceMap,
        breakpoints: &Breakpoints,
        effects: &[Effect],
    ) -> Self {
        let name = function.name;
        let branches = function
            .branches
            .into_iter()
            .map(|branch| {
                DebugBranch::new(
                    branch,
                    active_fragment,
                    fragments,
                    source_map,
                    breakpoints,
                    effects,
                )
            })
            .collect();

        Self { name, branches }
    }
}
