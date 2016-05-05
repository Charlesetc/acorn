// compiler/mod.rs

pub mod abstract_tree;

use self::abstract_tree::{AbstractTree, QBEBackend};
use utils::{Result, IR};

/// check_define ensures the tree passed to it is valid
/// for a define call
///
fn check_define<'a>(at: &'a mut AbstractTree) -> Result<()> {
    Ok(())
        .and_then(|_| at.check_length(3))
        .and_then(|_| at.check_argument_block(2))
}

fn compile_define(backend: &mut QBEBackend, tree: &mut AbstractTree) -> Result<IR> {

    let mut arguments_to_define = tree.arguments_mut();
    let mut block = arguments_to_define.pop().unwrap();

    let mut top_level_iterator = arguments_to_define.iter();
    top_level_iterator.next(); // get rid of the call to 'define'

    let name = top_level_iterator.next().unwrap().name();

    let mut arguments_to_block = block.arguments_mut();
    let mut block_expressions = arguments_to_block.pop().unwrap();

    let mut function_definition = format!("export function l ${}(", name);
    for argument in arguments_to_block {
        function_definition.push_str(&format!("l {},", argument.name()));
    }
    function_definition.push_str(") {");

    let ir = vec![function_definition, "@start".to_string()];

    block_expressions.arguments_mut()
                     .iter_mut()
                     .fold(Ok(ir), |acc, expression| {
                         acc.and_then(|mut ir| {
                             backend.compile_inner(expression).and_then(|mut new_ir| {
                                 ir.append(&mut new_ir);
                                 Ok(ir)
                             })
                         })
                     })
                     .and_then(|mut ir| {
                         ir.push("@end".to_string());
                         ir.push("ret %ret".to_string());
                         ir.push("}".to_string());
                         Ok(ir)
                     })
}

/// compile takes an abstract tree and compiles it - eventually
/// down to IR
pub fn compile<'a>(mut at: AbstractTree) -> Result<IR> {
    Ok(())
        .and_then(|_| at.match_symbol("define", check_define))
        .and_then(|_| at.assert_only_top_level("define"))
        .and_then(|_| {
            // compilation stage
            QBEBackend::new(at)
                .handle("define".to_string(), compile_define)
                .compile()
        })
}

#[cfg(test)]
mod tests {
    use utils::tests::{abstract_tree_item, assert_returns_error};
    use compiler::abstract_tree::AbstractTree;
    use compiler::abstract_tree::AbstractTree::*;
    use compiler::abstract_tree::TokenType::*;
    use utils::Position;
    use super::compile;

    fn construct_define_item<'a>(items: Vec<AbstractTree>) -> AbstractTree {
        abstract_tree_item(vec![
            Token(Symbol, "define".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
            Node(items, Position(0, 0)),
        ])
    }

    #[test]
    fn test_define_constraints() {
        // Test argument constraint
        let at = abstract_tree_item(vec![
            Token(Symbol, "define".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
        ]);
        assert_returns_error(compile(at), "define takes 2 arguments");

        // Test need for block constraint
        let at = abstract_tree_item(vec![
            Token(Symbol, "define".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
        ]);
        assert_returns_error(compile(at), "define expects a block for its 2th argument");

        // Test top level constraint
        let at = construct_define_item(vec![
                Token(Symbol, "block".to_string(), Position(0,0)),
                Node(vec![], Position(0,0)),
                Node(vec![construct_define_item(vec![
                    Token(Symbol, "block".to_string(), Position(0,0)),
                    Node(vec![], Position(0,0)),
                    Node(vec![], Position(0,0)),
                ])], Position(0,0)),
            ]);
        assert_returns_error(compile(at),
                             "define was invoked without being on the top level");
    }

    #[test]
    fn test_block_constraints() {
        let at = construct_define_item(vec![
            Token(Symbol, "block".to_string(), Position(0,0)),
        ]);
        assert_returns_error(compile(at), "block takes at least 1 arguments");

        let at = construct_define_item(vec![
            Token(Symbol, "block".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
        ]);
        assert_returns_error(compile(at),
                             "a block takes a list of arguments followed by a list of expressions");

        let at = construct_define_item(vec![
            Token(Symbol, "block".to_string(), Position(0,0)),
            Token(Int, "2".to_string(), Position(0,0)),
            Node(vec![Token(Int, "2".to_string(), Position(0,0))], Position(0,0)),
        ]);
        compile(at).ok().unwrap();
    }

}
