use crate::cp::{self, Functions};

pub fn define() -> anyhow::Result<Functions> {
    let mut functions = cp::Functions::new();

    let code = r#"
    "#;

    let data_stack = cp::execute(code.chars(), &mut functions)?;
    if !data_stack.is_empty() {
        anyhow::bail!("Defining functions left values on stack: {data_stack:?}")
    }

    Ok(functions)
}
