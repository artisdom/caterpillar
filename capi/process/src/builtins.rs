use crate::{CoreEffect, Function, Instructions, Stack, Value};

pub fn builtin(name: &str) -> Option<Builtin> {
    let builtin = match name {
        "add_i32" => add,
        "add_wrap_unsigned" => add_wrap_unsigned,
        "brk" => brk,
        "copy" => copy,
        "div" => div,
        "drop" => drop,
        "eq" => eq,
        "eval" => eval,
        "greater" => greater,
        "i32_to_i8" => i32_to_i8,
        "if" => if_,
        "mul" => mul,
        "neg" => neg,
        "remainder" => remainder,
        "sub" => sub,

        _ => {
            return None;
        }
    };

    Some(builtin)
}

pub type Builtin = fn(&mut Stack, &Instructions) -> Result;

fn add(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    let Some(c) = a.checked_add(b) else {
        return Err(CoreEffect::IntegerOverflow);
    };

    stack.push_operand(c);

    Ok(())
}

fn add_wrap_unsigned(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    let c = a.wrapping_add(b);
    let c = if c >= 0 { c } else { c - i32::MIN };

    stack.push_operand(c);

    Ok(())
}

fn brk(_: &mut Stack, _: &Instructions) -> Result {
    Err(CoreEffect::Breakpoint)
}

fn copy(stack: &mut Stack, _: &Instructions) -> Result {
    let a = stack.pop_operand()?;

    stack.push_operand(a);
    stack.push_operand(a);

    Ok(())
}

fn div(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    if b == 0 {
        return Err(CoreEffect::DivideByZero);
    }
    let Some(c) = a.checked_div(b) else {
        // Can't be divide by zero. Already handled that.
        return Err(CoreEffect::IntegerOverflow);
    };

    stack.push_operand(c);

    Ok(())
}

fn drop(stack: &mut Stack, _: &Instructions) -> Result {
    stack.pop_operand()?;
    Ok(())
}

fn eq(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let c = if a.0 == b.0 { 1 } else { 0 };

    stack.push_operand(c);

    Ok(())
}

fn eval(stack: &mut Stack, instructions: &Instructions) -> Result {
    let closure = stack.pop_operand()?;
    let closure = u32::from_le_bytes(closure.0);

    let (address, environment) = stack.closures.remove(&closure).unwrap();

    let mut arguments = Vec::new();
    for (name, value) in environment {
        arguments.push(name);
        stack.push_operand(value);
    }

    stack.push_frame(
        Function {
            arguments,
            first_instruction: address,
        },
        instructions,
    )?;

    Ok(())
}

fn greater(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    let c = if a > b { 1 } else { 0 };

    stack.push_operand(c);

    Ok(())
}

fn i32_to_i8(stack: &mut Stack, _: &Instructions) -> Result {
    let v = stack.pop_operand()?;

    let v = i32::from_le_bytes(v.0);
    let v: i8 = v.try_into()?;
    let [v] = v.to_le_bytes();
    let v = Value([v, 0, 0, 0]);

    stack.push_operand(v);

    Ok(())
}

fn if_(stack: &mut Stack, instructions: &Instructions) -> Result {
    let else_ = stack.pop_operand()?;
    let then = stack.pop_operand()?;
    let condition = stack.pop_operand()?;

    let (evaluate, discard) = if condition.0 == [0, 0, 0, 0] {
        (else_, then)
    } else {
        (then, else_)
    };

    // `eval` consumes the closure we evaluate, but we have to discard the other
    // one here, to no leak memory.
    let discard = u32::from_le_bytes(discard.0);
    stack.closures.remove(&discard);

    stack.push_operand(evaluate);
    eval(stack, instructions)
}

fn mul(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    let Some(c) = a.checked_mul(b) else {
        return Err(CoreEffect::IntegerOverflow);
    };

    stack.push_operand(c);

    Ok(())
}

fn neg(stack: &mut Stack, _: &Instructions) -> Result {
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);

    if a == i32::MIN {
        return Err(CoreEffect::IntegerOverflow);
    }
    let b = -a;

    stack.push_operand(b);

    Ok(())
}

fn remainder(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    if b == 0 {
        return Err(CoreEffect::DivideByZero);
    }
    let c = a % b;

    stack.push_operand(c);

    Ok(())
}

fn sub(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = i32::from_le_bytes(a.0);
    let b = i32::from_le_bytes(b.0);

    let Some(c) = a.checked_sub(b) else {
        return Err(CoreEffect::IntegerOverflow);
    };

    stack.push_operand(c);

    Ok(())
}

pub type Result = std::result::Result<(), CoreEffect>;
