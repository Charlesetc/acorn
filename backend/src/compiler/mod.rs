// compiler/mod.rs

#[derive(Debug)]
pub enum TokenType {
    Symbol,
    Str,
    Int,
    Float,
}

#[derive(Debug)]
pub enum AbstractTree<'a> {
    Node(Vec<AbstractTree<'a>>),
    Token(TokenType, &'a str),
}

pub fn compile<'a>(at: Vec<AbstractTree<'a>>) {
    println!("{:?}\n", at);
}
