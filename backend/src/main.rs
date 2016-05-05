// main.rs

mod compiler;
mod utils;
mod parser;


use compiler::abstract_tree::AbstractTree::*;
use compiler::abstract_tree::TokenType::*;
use utils::Position;
use utils::ErrorHandling;
use parser::parse;

fn main() {
    println!("{:?}", parse("(hi there)").unwrap());
}

fn main2() {
    let at = Node(vec![
        Node(vec![
            Token(Symbol, "define".to_string(), Position(1, 0)),
            Token(Symbol, "main".to_string(), Position(2, 0)),
            Node(vec![
                Token(Symbol, "block".to_string(), Position(3, 0)),
                // arguments to block
                Token(Int, "a".to_string(), Position(4, 0)),
                Node(vec![
                    Node(vec![
                        Token(Symbol, "add".to_string(), Position(6, 0)),
                        Token(Int, "2".to_string(), Position(7, 0)),
                        Token(Int, "2".to_string(), Position(8, 0))
                    ], Position(9, 0)),
                ], Position(10, 0))
            ], Position(11, 0)),
            ], Position(12, 0)),
        ],
                  Position(13, 0));
    let ir = compiler::compile(at).unpack_error();
    for line in ir {
        println!("{}", line)
    }
}
