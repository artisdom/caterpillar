use std::io::{self, stdout};

use crossterm::{
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use crate::language::{
    code::{Code, Expression, Fragment, Token},
    compiler::compile,
    host::Host,
    interpreter::Interpreter,
};

#[derive(Default)]
pub struct Editor {
    code: Code,
}

impl Editor {
    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn process_input(
        &mut self,
        line: String,
        host: &Host,
        interpreter: &mut Interpreter,
    ) {
        let mut command_and_arguments =
            line.trim().splitn(2, |ch: char| ch.is_whitespace());

        let Some(command) = command_and_arguments.next() else {
            return;
        };

        match command {
            command @ ":clear" => {
                let None = command_and_arguments.next() else {
                    println!("`{command}` command expects no arguments.");
                    return;
                };

                self.code = Code::default();
            }
            command @ ":insert" => {
                let Some(input_code) = command_and_arguments.next() else {
                    println!(
                        "`{command}` command expects input code as argument."
                    );
                    return;
                };

                compile(input_code, host, &mut self.code);
            }
            command @ ":reset" => {
                let None = command_and_arguments.next() else {
                    println!("`{command}` command expects no arguments.");
                    return;
                };

                *interpreter = Interpreter::default();
            }
            command => {
                println!("Unknown command: `{command}`");
            }
        }
    }

    pub fn render(
        &self,
        host: &Host,
        interpreter: &Interpreter,
    ) -> anyhow::Result<()> {
        render_code(&self.code, host, interpreter, stdout())?;
        Ok(())
    }
}

fn render_code(
    code: &Code,
    _: &Host,
    interpreter: &Interpreter,
    mut w: impl io::Write,
) -> anyhow::Result<()> {
    writeln!(w)?;

    for (i, fragment) in code.fragments.iter().enumerate() {
        if code.errors.contains(&i) {
            w.queue(SetForegroundColor(Color::Red))?;
        }

        if i == interpreter.next_fragment {
            w.queue(SetAttribute(Attribute::Bold))?;
            write!(w, " => ")?;
        } else {
            write!(w, "    ")?;
        }

        match fragment {
            Fragment::Expression { expression } => match expression {
                Expression::FunctionCall { target: _ } => {
                    writeln!(w, "call to host function (not supported yet)")?;
                }
                Expression::LiteralValue { value } => {
                    writeln!(w, "{value}")?;
                }
            },
            Fragment::UnexpectedToken { token } => {
                match token {
                    Token::Identifier { name } => {
                        write!(w, "{name}")?;
                    }
                    Token::LiteralNumber { value } => {
                        write!(w, "{value}")?;
                    }
                }

                writeln!(w, "    error: unexpected token")?;
            }
        }

        w.queue(ResetColor)?;
        w.queue(SetAttribute(Attribute::Reset))?;
    }

    if interpreter.next_fragment == code.fragments.len() {
        writeln!(w, " => ")?;
    }

    writeln!(w)?;
    write!(w, "{} > ", interpreter.state(code))?;

    w.flush()?;

    Ok(())
}
