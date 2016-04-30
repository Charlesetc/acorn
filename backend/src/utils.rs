// utils.rs

use std::result;
use std::process;
use std::io::Write;
use std::io;

// Position(line, column)
#[derive(Debug, Clone)]
pub struct Position(pub i32, pub i32);

// position: Position,
pub struct Error {
    pub description: String,
    pub position: Position,
}

pub trait ErrorHandling<T> {
    fn unpack_error(self) -> T;
}

pub type Result<T> = result::Result<T, Error>;

impl<T> ErrorHandling<T> for Result<T> {
    fn unpack_error(self) -> T {
        let mut stderr = &mut io::stderr();
        match self {
            Ok(a) => {
                writeln!(stderr, "compilation successful").unwrap();
                a
            }
            Err(ref error) => {
                writeln!(stderr, "compilation error:").unwrap();
                writeln!(stderr, "\tline {} column {}", error.position.0, error.position.1).unwrap();
                writeln!(stderr, "\t{}", error.description).unwrap();
                process::exit(0)
            }
        }
    }
}
