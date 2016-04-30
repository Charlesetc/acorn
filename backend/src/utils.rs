// utils.rs

use std::result;

// position: Position,
pub struct Error { pub description: String }

pub trait ErrorHandling {
    fn handle_error(&self) -> ();
}

pub type Result<T> = result::Result<T, Error>;

impl<T> ErrorHandling for Result<T> {
    fn handle_error(&self) {
        match self {
            &Ok(_) => {}
            &Err(ref error) => {
                println!("compilation error: {}", error.description)
            }
        }
    }
}
