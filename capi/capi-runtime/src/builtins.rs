use crate::{data_stack::StackUnderflow, Value};

use super::data_stack::DataStack;

pub fn add(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_add(b.0) else {
        return Err(BuiltinError::IntegerOverflow);
    };

    data_stack.push(c);

    Ok(None)
}

pub fn copy(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    let i = i.0;

    data_stack.save(i)?;
    let a = data_stack.clone()?;
    data_stack.restore();

    data_stack.push(a);

    Ok(None)
}

pub fn drop(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    let i = i.0;

    data_stack.save(i)?;
    data_stack.pop()?;
    data_stack.restore();

    Ok(None)
}

pub fn load(data_stack: &mut DataStack) -> Result {
    let address = data_stack.pop()?;

    let address = address.0;

    Ok(Some(BuiltinEffect::Load { address }))
}

pub fn mul(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_mul(b.0) else {
        return Err(BuiltinError::IntegerOverflow);
    };

    data_stack.push(c);

    Ok(None)
}

pub fn place(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;
    let a = data_stack.pop()?;

    let i = i.0;

    data_stack.save(i)?;
    data_stack.push(a);
    data_stack.restore();

    Ok(None)
}

pub fn remainder(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    if b.0 == 0 {
        return Err(BuiltinError::DivideByZero);
    }
    let c = a.0 % b.0;

    data_stack.push(c);

    Ok(None)
}

pub fn store(data_stack: &mut DataStack) -> Result {
    let address = data_stack.pop()?;
    let value = data_stack.pop()?;

    let address = address.0;

    Ok(Some(BuiltinEffect::Store { address, value }))
}

pub fn sub(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_sub(b.0) else {
        return Err(BuiltinError::IntegerOverflow);
    };

    data_stack.push(c);

    Ok(None)
}

pub fn submit_frame() -> Result {
    Ok(Some(BuiltinEffect::SubmitFrame))
}

pub fn take(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    let i = i.0;

    data_stack.save(i)?;
    let a = data_stack.pop()?;
    data_stack.restore();

    data_stack.push(a);

    Ok(None)
}

pub fn write_tile(data_stack: &mut DataStack) -> Result {
    let value = data_stack.pop()?;
    let y = data_stack.pop()?;
    let x = data_stack.pop()?;

    let effect = BuiltinEffect::SetTile {
        x: x.0,
        y: y.0,
        value: value.0,
    };

    data_stack.push(x);
    data_stack.push(y);

    Ok(Some(effect))
}

pub type Result = std::result::Result<Option<BuiltinEffect>, BuiltinError>;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum BuiltinEffect {
    Error(BuiltinError),

    Load { address: u8 },
    Store { address: u8, value: Value },

    SetTile { x: u8, y: u8, value: u8 },
    SubmitFrame,
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    thiserror::Error,
)]
pub enum BuiltinError {
    #[error("Divide by zero")]
    DivideByZero,

    #[error("Integer overflow")]
    IntegerOverflow,

    #[error(transparent)]
    StackUnderflow(#[from] StackUnderflow),
}
