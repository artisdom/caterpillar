use std::collections::BTreeMap;

use crate::{
    code::{
        search::Find, Changes, FunctionInUpdate, FunctionUpdate, NamedFunctions,
    },
    hash::Hash,
};

pub fn detect_changes(
    old_functions: Option<NamedFunctions>,
    new_functions: &NamedFunctions,
) -> Changes {
    let old_functions = old_functions.unwrap_or_default();

    let mut added = BTreeMap::new();
    let mut updated = Vec::new();

    for (new_index, new_function) in new_functions {
        if old_functions
            .find_by_hash(&Hash::new(new_function))
            .is_some()
        {
            // Function has not changed. We can forget about it.
            continue;
        }

        let name = new_function
            .name
            .as_deref()
            .expect("Named function should have a name.");
        if let Some(Find {
            find: old_function,
            metadata: old_index,
        }) = old_functions.find_by_name(name)
        {
            // Found a function with the same name. But it can't have the same
            // hash, or we wouldn't have made it here. Assuming the new function
            // is an updated version of the old.
            updated.push(FunctionUpdate {
                old: FunctionInUpdate {
                    index: old_index,
                    function: old_function,
                },
                new: FunctionInUpdate {
                    index: *new_index,
                    function: new_function.clone(),
                },
            });

            continue;
        }

        // If we make it here, there was neither an identical function before,
        // nor one with the same name. This must mean this function is new.
        added.insert(*new_index, new_function.clone());
    }

    Changes { added, updated }
}
