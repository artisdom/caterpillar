use crate::data_stack::StackUnderflow;

use super::data_stack::DataStack;

pub fn add(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_add(b.0) else {
        return Err(Error::IntegerOverflow);
    };

    data_stack.push(c);

    Ok(None)
}

pub fn copy(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    data_stack.save(i.0);
    let a = data_stack.clone()?;
    data_stack.restore();

    data_stack.push(a);

    Ok(None)
}

pub fn drop(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    data_stack.save(i.0);
    data_stack.pop()?;
    data_stack.restore();

    Ok(None)
}

pub fn mul(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_mul(b.0) else {
        return Err(Error::IntegerOverflow);
    };

    data_stack.push(c);

    Ok(None)
}

pub fn place(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;
    let a = data_stack.pop()?;

    data_stack.save(i.0);
    data_stack.push(a);
    data_stack.restore();

    Ok(None)
}

pub fn store(data_stack: &mut DataStack, mem: &mut [u8]) -> Result {
    let value = data_stack.pop()?;
    let addr = data_stack.pop()?;

    let index: usize = addr.0.into();
    mem[index] = value.0;

    data_stack.push(addr);

    Ok(None)
}

pub fn sub(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_sub(b.0) else {
        return Err(Error::IntegerOverflow);
    };

    data_stack.push(c);

    Ok(None)
}

pub fn take(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    data_stack.save(i.0);
    let a = data_stack.pop()?;
    data_stack.restore();

    data_stack.push(a);

    Ok(None)
}

pub fn tile(data_stack: &mut DataStack, _mem: &mut [u8]) -> Result {
    let value = data_stack.pop()?;
    let y = data_stack.pop()?;
    let x = data_stack.pop()?;

    let effect = Effect::SetTile {
        x: x.0,
        y: y.0,
        value: value.0,
    };

    data_stack.push(x);
    data_stack.push(y);

    Ok(Some(effect))
}

pub type Result = std::result::Result<Option<Effect>, Error>;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Effect {
    Error(Error),
    SetTile { x: u8, y: u8, value: u8 },
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
pub enum Error {
    #[error("Integer overflow")]
    IntegerOverflow,

    #[error(transparent)]
    StackUnderflow(#[from] StackUnderflow),
}
