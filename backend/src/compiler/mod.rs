// compiler/mod.rs

pub mod abstract_tree;

use self::abstract_tree::AbstractTree;
use utils::Result;

/// check_define ensures the tree passed to it is valid
/// for a define call
///
fn check_define(at: &mut AbstractTree) -> Result<()> {
    Ok(())
        .and_then(|_| at.check_length(3))
        .and_then(|_| at.check_argument_block(2))
        .and_then(|_| at.assert_only_top_level("define"))
}

/// compile takes an abstract tree and compiles it - eventually
/// down to a rust String.
///
pub fn compile<'a>(mut at: AbstractTree<'a>) -> Result<()> {
    Ok(()).and_then(|_| at.match_symbol("define", check_define))
}

#[cfg(test)]
mod tests {
    use utils::tests::{abstract_tree_item, assert_returns_error};
    use compiler::abstract_tree::AbstractTree;
    use compiler::abstract_tree::AbstractTree::*;
    use compiler::abstract_tree::TokenType::*;
    use utils::Position;
    use super::compile;

    fn construct_define_item<'a>(items: Vec<AbstractTree<'a>>) -> AbstractTree<'a> {
        abstract_tree_item(vec![
            Token(Symbol, "define", Position(0,0)),
            Token(Int, "2", Position(0,0)),
            Node(items, Position(0, 0)),
        ])
    }

    #[test]
    fn test_define_constraints() {
        // Test argument constraint
        let at = abstract_tree_item(vec![
            Token(Symbol, "define", Position(0,0)),
            Token(Int, "2", Position(0,0)),
        ]);
        assert_returns_error(compile(at), "define takes 2 arguments");

        // Test need for block constraint
        let at = abstract_tree_item(vec![
            Token(Symbol, "define", Position(0,0)),
            Token(Int, "2", Position(0,0)),
            Token(Int, "2", Position(0,0)),
        ]);
        assert_returns_error(compile(at), "define expects a block for its 2th argument");

        // Test top level constraint
        let at = construct_define_item(vec![
                Token(Symbol, "block", Position(0,0)),
                Node(vec![], Position(0,0)),
                Node(vec![construct_define_item(vec![
                    Token(Symbol, "block", Position(0,0)),
                    Node(vec![], Position(0,0)),
                    Node(vec![], Position(0,0)),
                ])], Position(0,0)),
            ]);
        assert_returns_error(compile(at), "define was invoked without being on the top level");
    }

    #[test]
    fn test_block_constraints() {
        let at = construct_define_item(vec![
            Token(Symbol, "block", Position(0,0)),
        ]);
        assert_returns_error(compile(at), "block takes 2 arguments");

        let at = construct_define_item(vec![
            Token(Symbol, "block", Position(0,0)),
            Token(Int, "2", Position(0,0)),
            Token(Int, "2", Position(0,0)),
        ]);
        assert_returns_error(compile(at),
                             "a block takes a list of arguments followed by a list of expressions");

        let at = construct_define_item(vec![
            Token(Symbol, "block", Position(0,0)),
            Node(vec![Token(Int, "2", Position(0,0))], Position(0,0)),
            Node(vec![Token(Int, "2", Position(0,0))], Position(0,0)),
        ]);
        compile(at).ok().unwrap();
    }

}
