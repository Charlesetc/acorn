// main.rs

mod compiler;
mod utils;
mod parser;

use utils::ErrorHandling;

fn main() {
    let source = "define start { print_number }";
    let abstract_tree = parser::parse(source)
                            .unpack_error()
                            .expect("failed to parse anything, weird");
    let ir = compiler::compile(abstract_tree).unpack_error();
    for line in ir {
        println!("{}", line);
    }
}
