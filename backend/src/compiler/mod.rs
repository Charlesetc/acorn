// compiler/mod.rs

pub mod abstract_tree;

use self::abstract_tree::AbstractTree;

fn check_define(at: &mut AbstractTree) {
    println!("check define {:?}\n", at);
}

pub fn compile<'a>(mut at: AbstractTree<'a>) {
    at.visit_after(check_define);
    at.match_symbol("define", check_define);
}
