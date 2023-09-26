use std::{thread, time::Duration};

use capi_core::{value, Context, DataStackResult, Evaluator};

pub fn delay_ms(
    evaluator: &mut Evaluator,
    _: &mut Context,
) -> DataStackResult<()> {
    let (delay_ms, _) = evaluator.data_stack.pop_specific::<value::Number>()?;
    thread::sleep(Duration::from_millis(delay_ms.0.try_into().unwrap()));
    Ok(())
}

pub fn print(
    evaluator: &mut Evaluator,
    _: &mut Context,
) -> DataStackResult<()> {
    let value = evaluator.data_stack.pop_any()?;
    println!("{}", value.kind);
    evaluator.data_stack.push(value);
    Ok(())
}
