use crate::{Effect, Stack};

pub fn builtin_by_name(name: &str) -> Option<Builtin> {
    let builtin = match name {
        "sub_u8_wrap" => sub_u8_wrap,

        _ => {
            return None;
        }
    };

    Some(builtin)
}

pub type Builtin = fn(&mut Stack) -> Result;

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
