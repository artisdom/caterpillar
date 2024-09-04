use capi_compiler::{
    fragments::{self, FragmentId, Fragments},
    source_map::SourceMap,
};
use capi_process::Process;

use super::Branch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugFunction {
    pub name: Option<String>,
    pub branches: Vec<Branch>,
    pub active_fragment: Option<FragmentId>,
}

impl DebugFunction {
    pub fn new(
        function: fragments::Function,
        active_fragment: Option<FragmentId>,
        fragments: &Fragments,
        source_map: &SourceMap,
        process: &Process,
    ) -> Self {
        let name = function.name;
        let branches = function
            .branches
            .into_iter()
            .map(|branch| {
                Branch::new(
                    branch,
                    active_fragment,
                    fragments,
                    source_map,
                    process,
                )
            })
            .collect();

        Self {
            name,
            branches,
            active_fragment,
        }
    }
}
