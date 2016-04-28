// main.rs

mod compiler;

use compiler::AbstractTree::*;
use compiler::TokenType::*;

fn main() {
    let at = Node(vec![
                Node(vec![Token(Symbol, "add"), Token(Int, "2"), Token(Int, "2")]),
                Node(vec![Token(Symbol, "define"), Token(Int, "a"), Token(Int, "2")]),
            ]);
    compiler::compile(at);
}
