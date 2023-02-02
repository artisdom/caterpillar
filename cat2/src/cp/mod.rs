mod stack;

pub use self::stack::{Stack, Value};

pub fn interpret(code: &str, stack: &mut Stack) {
    let tokens = tokenize(code);
    evaluate(tokens, stack);
}

fn tokenize(code: &str) -> impl Iterator<Item = &str> {
    code.split_whitespace()
}

fn evaluate<'a>(tokens: impl Iterator<Item = &'a str>, stack: &mut Stack) {
    for token in tokens {
        match token {
            "clone" => {
                let value = stack.pop_any();

                stack.push(value.clone());
                stack.push(value);
            }
            "or" => {
                let b = stack.pop_bool();
                let a = stack.pop_bool();

                let value = Value::Bool(a || b);
                stack.push(value);
            }
            "swap" => {
                let b = stack.pop_any();
                let a = stack.pop_any();

                stack.push(b);
                stack.push(a);
            }
            "=" => {
                let Some(Value::U8(b)) = stack.pop() else {
                    panic!("Expected `u8`")
                };
                let Some(Value::U8(a)) = stack.pop() else {
                    panic!("Expected `u8`")
                };

                let value = Value::Bool(a == b);
                stack.push(value);
            }
            token => {
                if let Ok(value) = token.parse::<u8>() {
                    stack.push(Value::U8(value));
                    continue;
                }

                panic!("Unknown token: `{token}`")
            }
        }
    }
}
