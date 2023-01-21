use std::{cell::RefCell, rc::Rc};

pub fn init() -> (Interpreter, Output) {
    let background_color = Rc::new(RefCell::new([0., 0., 0., 1.]));
    let language = Interpreter {
        background_color: background_color.clone(),
    };

    (language, background_color)
}

pub struct Interpreter {
    background_color: Rc<RefCell<[f64; 4]>>,
}

impl Interpreter {
    pub fn interpret(&self, code: &str) {
        let code = code.chars();

        let value = parse_color_channel(code);

        if let Some(value) = value {
            *self.background_color.borrow_mut() = [value, value, value, 1.];
        }
    }
}

fn parse_color_channel(code: impl Iterator<Item = char>) -> Option<f64> {
    let word = code.take_while(|c| !c.is_whitespace()).collect::<String>();

    let Ok(value) = word.parse::<u8>() else {
        return None;
    };
    Some(value as f64 / u8::MAX as f64)
}

pub type Output = Rc<RefCell<[f64; 4]>>;
