mod action;
mod character;
mod modifier;

pub use action::*;
pub use character::*;

#[derive(Debug)]
pub enum Error {
    Undefined,
    ToDo,
}
