use super::{evaluator::FunctionNotFound, DataStack, Functions};

pub type Builtin =
    fn(&Functions, &mut DataStack) -> Result<(), FunctionNotFound>;

pub fn get(name: &str) -> Option<Builtin> {
    let builtin = match name {
        "clone" => clone,
        "drop" => drop,
        "min" => min,
        "or" => or,
        "swap" => swap,
        "=" => eq,
        "+" => add,
        "-" => sub,
        _ => return None,
    };

    Some(builtin)
}

fn add(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let b = data_stack.pop_u8();
    let a = data_stack.pop_u8();

    let x = a.saturating_add(b);

    data_stack.push(x);

    Ok(())
}

fn clone(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let value = data_stack.pop_any();

    data_stack.push(value.clone());
    data_stack.push(value);

    Ok(())
}

fn drop(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    data_stack.pop_any();

    Ok(())
}

fn eq(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let b = data_stack.pop_u8();
    let a = data_stack.pop_u8();

    data_stack.push(a == b);

    Ok(())
}

fn min(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let b = data_stack.pop_u8();
    let a = data_stack.pop_u8();

    let x = u8::min(a, b);

    data_stack.push(x);

    Ok(())
}

fn or(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let b = data_stack.pop_bool();
    let a = data_stack.pop_bool();

    data_stack.push(a || b);

    Ok(())
}

fn sub(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let b = data_stack.pop_u8();
    let a = data_stack.pop_u8();

    let x = a.saturating_sub(b);

    data_stack.push(x);

    Ok(())
}

fn swap(
    _: &Functions,
    data_stack: &mut DataStack,
) -> Result<(), FunctionNotFound> {
    let b = data_stack.pop_any();
    let a = data_stack.pop_any();

    data_stack.push(b);
    data_stack.push(a);

    Ok(())
}
