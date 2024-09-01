use crate::{value::IntegerOverflow, Effect, Stack};

pub fn builtin_by_name(name: &str) -> Option<Builtin> {
    let builtin = match name {
        "not" => not,
        "remainder_i32" => remainder_i32,
        "sub_i32" => sub_i32,
        "sub_u8" => sub_u8,
        "sub_u8_wrap" => sub_u8_wrap,

        _ => {
            return None;
        }
    };

    Some(builtin)
}

pub type Builtin = fn(&mut Stack) -> Result;

fn not(stack: &mut Stack) -> Result {
    let a = stack.pop_operand()?;

    let b = if a.0 == [0; 4] { 1 } else { 0 };
    stack.push_operand(b);

    Ok(())
}

fn remainder_i32(stack: &mut Stack) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    if b == 0 {
        return Err(Effect::DivideByZero);
    }
    let c = a % b;

    stack.push_operand(c);

    Ok(())
}

fn sub_i32(stack: &mut Stack) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    let Some(c) = a.checked_sub(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn sub_u8(stack: &mut Stack) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let Some(c) = a.checked_sub(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn sub_u8_wrap(stack: &mut Stack) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let c = a.wrapping_sub(b);
    stack.push_operand(c);

    Ok(())
}

pub type Result = std::result::Result<(), Effect>;
