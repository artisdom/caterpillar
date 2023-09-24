use std::sync::mpsc::{Receiver, RecvError, TryRecvError};

use crate::pipeline::PipelineError;

use super::{
    evaluator::{EvaluatorError, EvaluatorState},
    interpreter::Interpreter,
};

pub fn start(
    code: &str,
    updates: Receiver<String>,
) -> Result<(), RuntimeError> {
    let mut interpreter = Interpreter::new(code)?;

    loop {
        let new_code = match interpreter.step()? {
            EvaluatorState::InProgress => match updates.try_recv() {
                Ok(new_code) => Some(new_code),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    return Err(RuntimeError::UpdatesDisconnected);
                }
            },
            EvaluatorState::Finished => {
                eprintln!();
                eprintln!("> Program finished.");
                eprintln!("  > will restart on change to script");
                eprintln!("  > press CTRL-C to abort");
                eprintln!();

                match updates.recv() {
                    Ok(new_code) => Some(new_code),
                    Err(RecvError) => {
                        return Err(RuntimeError::UpdatesDisconnected);
                    }
                }
            }
        };

        if let Some(new_code) = new_code {
            interpreter.update(&new_code)?;
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Pipeline error")]
    Pipeline(#[from] PipelineError),

    #[error("Evaluator error")]
    Evaluator(#[from] EvaluatorError),

    #[error("Update channel is disconnected")]
    UpdatesDisconnected,
}
