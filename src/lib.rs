mod action;
mod character;

pub use action::*;
pub use character::*;

#[derive(Debug)]
pub enum Error {
    Undefined,
    ToDo,
}
