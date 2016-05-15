// compiler/mod.rs

pub mod abstract_tree;
pub mod backend;

use self::abstract_tree::AbstractTree;
use self::backend::LLVMBackend;
use utils::{Result, IR};

/// check_define ensures the tree passed to it is valid
/// for a define call
///
fn check_define<'a>(at: &'a mut AbstractTree) -> Result<()> {
    Ok(())
        .and_then(|_| at.check_length(3))
        .and_then(|_| at.check_argument_block(2))
}


// Maybe this should be put in the backend - or it's own module.
fn compile_define(backend: &mut LLVMBackend, tree: &mut AbstractTree) -> Result<IR> {
    backend.start_stack();

    let mut arguments_to_define = tree.arguments_mut();
    let mut block = arguments_to_define.pop().unwrap();

    let mut top_level_iterator = arguments_to_define.iter();
    top_level_iterator.next(); // get rid of the call to 'define'

    let name = top_level_iterator.next().unwrap().name();

    let mut arguments_to_block = block.arguments_mut();

    let mut block_expressions = arguments_to_block.pop().unwrap();
    let mut arguments_to_block = arguments_to_block.iter_mut();

    // get rid of 'block', the first argument to block.
    arguments_to_block.next();

    let mut function_definition = format!("define %object @{}(", name);
    let mut beginning = true;

    let mut argument_ir = vec![];
    let mut i = 0;
    for argument in arguments_to_block {
        if !beginning {
            function_definition.push(',')
        }

        function_definition.push_str(&format!("%object %in_arg.{}", i));
        argument_ir.append(&mut backend.set_var_ir(argument.name(), format!("in_arg.{}", i)));
        backend.add_assignee(argument.name());

        // comma formatting (facepalm)
        beginning = false;

        i += 1;
    }

    function_definition.push_str(") {");
    let mut ir = vec![function_definition];

    let ir_result = block_expressions.arguments_mut()
                                     .iter_mut()
                                     .fold(Ok(vec![]), |acc, expression| {
                                         acc.and_then(|mut ir| {
                                             backend.compile_inner(expression)
                                                    .and_then(|mut new_ir| {
                                                        ir.append(&mut new_ir);
                                                        Ok(ir)
                                                    })
                                         })
                                     })
                                     .and_then(|mut ir| {
                                         ir.push(format!("ret %object %{}",
                                                         backend.get_counter("ret")));
                                         ir.push("}".to_string());
                                         Ok(ir)
                                     });

    let stack = backend.end_stack();
    let mut ir_from_stack = stack.values()
        .map(|assignee| format!("%{} = alloca %object", assignee.name))
        .collect::<Vec<_>>();
    ir_result.and_then(|mut r| {
        ir.append(&mut ir_from_stack);
        ir.append(&mut argument_ir);
        ir.append(&mut r);
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
            LLVMBackend::new(at)
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
