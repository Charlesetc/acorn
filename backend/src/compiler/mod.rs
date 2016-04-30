// compiler/mod.rs

pub mod abstract_tree;

use self::abstract_tree::{
    AbstractTree,
};
use utils::Result;

fn check_define(at: &mut AbstractTree) -> Result {
    at.check_length(2)
}

pub fn compile<'a>(mut at: AbstractTree<'a>) {
    at.match_symbol("define", check_define);
}
