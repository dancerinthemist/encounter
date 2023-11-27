mod action;
pub mod character;

pub use action::*;
pub use character::*;
pub use serde;
pub use serde_json;
use std::fmt::{Display, Formatter};
use thiserror::Error;
pub use typetag;

#[derive(Debug, Error)]
pub enum Error {
    Undefined,
    ToDo,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}
