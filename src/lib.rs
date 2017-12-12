#[macro_use]
extern crate lazy_static;

mod result;

pub mod ast;

pub mod functions;
pub mod object;
pub mod env;

pub mod scanner;
pub mod parser;
pub mod interpreter;
pub mod resolver;

pub use result::{Result, Error};

/// Boxer converts a type into its Boxed form
pub trait Boxer {
    /// Convert to a boxed version
    fn boxed(self) -> Box<Self>;
}
