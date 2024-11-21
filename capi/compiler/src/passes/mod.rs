mod build_call_graph;
mod detect_changes;
mod generate_instructions;
mod resolve_non_recursive_functions;
mod resolve_recursive_calls;
mod resolve_recursive_local_functions;

pub use {
    build_call_graph::order_functions_by_dependencies,
    detect_changes::detect_changes,
    generate_instructions::generate_instructions,
    resolve_non_recursive_functions::resolve_non_recursive_functions,
    resolve_recursive_calls::resolve_recursive_calls,
    resolve_recursive_local_functions::resolve_recursive_local_functions,
};
