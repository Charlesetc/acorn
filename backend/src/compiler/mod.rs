// compiler/mod.rs

pub mod abstract_tree;

use self::abstract_tree::{
    AbstractTree,
};
use utils::Result;

fn check_define(at: &mut AbstractTree) -> Result<()> {
    Ok(())
        .and_then(|_| at.check_length(3))
}

pub fn compile<'a>(mut at: AbstractTree<'a>) -> Result<()> {
    Ok(())
        .and_then(|_| at.match_symbol("define", check_define))
}
