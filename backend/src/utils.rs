// utils.rs

use std::result;
use std::process;
use std::io::Write;
use std::io;

// Position(line, column)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position(pub i32, pub i32);

// position: Position,
#[derive(Debug, Clone)]
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
        ], Position(0, 2))
    }

    pub fn abstract_tree_item<'a>(at: Vec<AbstractTree<'a>>) -> AbstractTree {
        Node(vec![Node(at, Position(0,0))], Position(0,0))
    }

    pub fn assert_returns_error(result: Result<()>, description: &str) {
        assert_eq!(result.err().unwrap().description, description)
    }

}
