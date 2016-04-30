// main.rs

mod compiler;
mod utils;


use compiler::abstract_tree::AbstractTree::*;
use compiler::abstract_tree::TokenType::*;
use utils::ErrorHandling;

fn main() {
    let at = Node(vec![
                Node(vec![Token(Symbol, "add"), Token(Int, "2"), Token(Int, "2")]),
                Node(vec![Token(Symbol, "define"), Token(Int, "a"), Token(Int, "2")]),
            ]);
    compiler::compile(at).handle_error();
    println!("compilation successful")
}
