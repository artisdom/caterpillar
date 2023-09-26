include!(concat!(env!("OUT_DIR"), "/script.rs"));

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    if let Err(err) = main_inner() {
        panic!("Error: {err:?}");
    }
}

fn main_inner() -> anyhow::Result<()> {
    let mut interpreter = capi_core::Interpreter::new(SCRIPT)?;
    interpreter
        .register_platform([("print", print as capi_core::NativeFunction)]);

    while !interpreter.step()?.finished() {}

    Ok(())
}

fn print(
    evaluator: &mut capi_core::Evaluator,
    _: &mut capi_core::Context,
) -> Result<(), capi_core::runtime::data_stack::DataStackError> {
    let value = evaluator.data_stack.pop_any()?;
    tracing::info!("{}", value.kind);
    evaluator.data_stack.push(value);
    Ok(())
}
