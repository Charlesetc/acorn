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
                     Token(Symbol, "add", Position(0, 0)),
                     Token(Int, "2", Position(1, 0)),
                     Token(Int, "2", Position(2, 0))
                ], Position(3, 0)),
                Node(vec![
                     Token(Symbol, "define", Position(4, 0)),
                     Token(Int, "a", Position(5, 0)),
                     Node(vec![
                          Token(Symbol, "block", Position(6, 0)),
                          Node(vec![Token(Int, "a", Position(7, 0))], Position(8, 0)),
                          Node(vec![Token(Int, "2", Position(9, 0))], Position(10, 0)),
                     ], Position(11, 0)),
                ], Position(12, 0)),
            ],
             Position(0, 0));
    let ir = compiler::compile(at).unpack_error();
    for line in ir {
        println!("{}", line)
    }
}
