// main.rs

mod compiler;
mod utils;


use compiler::abstract_tree::AbstractTree::*;
use compiler::abstract_tree::TokenType::*;
use utils::Position;
use utils::ErrorHandling;

fn main() {
    let at = Node(vec![
                Node(vec![
                     Token(Symbol, "add", Position(0, 2)),
                     Token(Int, "2", Position(0, 2)),
                     Token(Int, "2", Position(0, 2))
                ], Position(0, 2)),
                Node(vec![
                     Token(Symbol, "define", Position(0, 2)),
                     Token(Int, "a", Position(0, 2)),
                     Token(Int, "2", Position(0, 2))
                ], Position(0, 2)),
            ], Position(0, 2));
    compiler::compile(at).unpack_error()
}
