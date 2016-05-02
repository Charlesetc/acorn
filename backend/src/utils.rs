// utils.rs

use std::result;
use std::process;
use std::io::Write;
use std::io;

/// Represents a line and column number
///
/// Position(line: i64, column: i64)
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position(pub i64, pub i64);

/// Represents an error - consisting of
/// a description and a position.
///
/// Position(line: i64, column: i64)
///
#[derive(Debug, Clone)]
pub struct Error {
    pub description: String,
    pub position: Position,
}

/// The compiled form of acorn:
/// each item in the vector should
/// represent one line of qbe ir.
pub type IR = Vec<String>;


/// This is used to print any errors
/// that were found in compilation.
///
/// If there are errors, the process exits.
///
pub trait ErrorHandling<T> {
    fn unpack_error(self) -> T;
}

impl<'a, T> ErrorHandling<T> for Result<T> {
    fn unpack_error(self) -> T {
        let mut stderr = &mut io::stderr();
        match self {
            Ok(a) => {
                writeln!(stderr, "compilation successful").unwrap();
                a
            }
            Err(ref error) => {
                writeln!(stderr, "compilation error:").unwrap();
                writeln!(stderr,
                         "\tline {} column {}",
                         error.position.0,
                         error.position.1)
                    .unwrap();
                writeln!(stderr, "\t{}", error.description).unwrap();
                process::exit(0)
            }
        }
    }
}

///
/// The result type used in the compiler
///
pub type Result<T> = result::Result<T, Error>;

#[cfg(test)]
pub mod tests {
    use compiler::abstract_tree::AbstractTree;
    use compiler::abstract_tree::AbstractTree::*;
    use compiler::abstract_tree::TokenType::*;
    use utils::Position;
    use utils::Result;

    pub fn generate_data<'a>() -> AbstractTree<'a> {
        Node(vec![
            Node(vec![
                 Token(Symbol, "foo", Position(0,0)),
                 Token(Int, "2", Position(0,0)),
                 Token(Int, "2", Position(0,0)),
                 Node(vec![
                        Token(Symbol, "foo", Position(0,0)),
                        Token(Symbol, "foo", Position(0,0)),
                ], Position(0, 2)),
            ], Position(0, 2)),
            Node(vec![
                 Token(Symbol, "define", Position(0,0)),
                 Token(Int, "2", Position(0,0))
            ], Position(0, 2)),
        ],
             Position(0, 2))
    }

    pub fn abstract_tree_item<'a>(at: Vec<AbstractTree<'a>>) -> AbstractTree {
        Node(vec![Node(at, Position(0, 0))], Position(0, 0))
    }

    pub fn assert_returns_error<T>(result: Result<T>, description: &str) {
        assert_eq!(result.err().unwrap().description, description)
    }

}


pub fn err_position<T>(position: Position, description: String) -> Result<T> {
    Err(Error {
        description: description,
        position: position,
    })
}
