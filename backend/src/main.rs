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
            Token(Symbol, "define", Position(1, 0)),
            Token(Symbol, "main", Position(2, 0)),
            Node(vec![
                Token(Symbol, "block", Position(3, 0)),
                Node(vec![ // arguments to block
                    Token(Int, "a", Position(4, 0))
                ], Position(5, 0)),
                Node(vec![
                    Node(vec![
                        Token(Symbol, "add", Position(6, 0)),
                        Token(Int, "2", Position(7, 0)),
                        Token(Int, "2", Position(8, 0))
                    ], Position(9, 0)),
                ], Position(10, 0))
            ], Position(11, 0)),
            ], Position(12, 0)),
        ], Position(13, 0));
    let ir = compiler::compile(at).unpack_error();
    for line in ir {
        println!("{}", line)
    }
}
