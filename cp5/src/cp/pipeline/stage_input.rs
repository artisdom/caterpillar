use std::collections::VecDeque;

use crate::cp::syntax::{SyntaxElement, SyntaxTree};

#[derive(Debug)]
pub struct StageInput<T> {
    elements: VecDeque<T>,
}

impl<T> StageInput<T> {
    pub fn new() -> Self {
        Self {
            elements: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn add(&mut self, element: T) {
        self.elements.push_back(element)
    }

    pub fn reader(&mut self) -> StageInputReader<T> {
        StageInputReader {
            inner: self,
            num_read: 0,
        }
    }
}

impl<T> FromIterator<T> for StageInput<T> {
    fn from_iter<I: IntoIterator<Item = T>>(elements: I) -> Self {
        Self {
            elements: elements.into_iter().collect(),
        }
    }
}

impl From<SyntaxTree> for StageInput<SyntaxElement> {
    fn from(syntax_tree: SyntaxTree) -> Self {
        Self {
            elements: syntax_tree.elements.into(),
        }
    }
}

#[derive(Debug)]
pub struct StageInputReader<'r, T> {
    inner: &'r mut StageInput<T>,
    num_read: usize,
}

impl<'r, T> StageInputReader<'r, T> {
    pub fn peek(&self) -> Result<&T, NoMoreInput> {
        self.inner.elements.get(self.num_read).ok_or(NoMoreInput)
    }

    pub fn next(&mut self) -> Result<&T, NoMoreInput> {
        let element =
            self.inner.elements.get(self.num_read).ok_or(NoMoreInput)?;
        self.num_read += 1;
        Ok(element)
    }

    pub fn take(&mut self) {
        let _ = self.inner.elements.drain(..self.num_read).last();
    }
}

#[derive(Debug, thiserror::Error)]
#[error("No more input")]
pub struct NoMoreInput;
