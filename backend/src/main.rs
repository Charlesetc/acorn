// main.rs

#[macro_use(puts)]
extern crate puts;

mod compiler;
mod utils;
mod parser;

use utils::ErrorHandling;

fn main() {
    let source =
"define start { x
    print_number x
}";

    let abstract_tree = parser::parse(source)
                            .unpack_error()
                            .expect("failed to parse anything, weird");
    // puts!(abstract_tree);
    // return;
    let ir = compiler::compile(abstract_tree).unpack_error();
    for line in ir {
        println!("{}", line);
    }
}
