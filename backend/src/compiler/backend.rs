// compiler/backend.rs

use std::collections::HashMap;

use utils::{Result, IR, Position, err_position};
use compiler::abstract_tree::{AbstractTree, TokenType};
use compiler::abstract_tree::AbstractTree::*;

mod utils {
    pub fn generate_function_arguments(i: usize) -> String {
        let mut output = "(".to_string();
        for x in 0..(i - 1) {
            output.push_str(&format!("%object arg{}, ", x));
        }
        output.push_str(")");
        output
    }
}

/// A struct holding information
/// about a variable that has been
/// assigned to. It has a name for the
/// variable and a location
#[derive(Debug)]
struct Assignee {
    name: String,
    position: Position,
}

pub struct LLVMBackend {
    pub abstract_tree: Option<AbstractTree>,
    pub transformations: HashMap<String, fn(&mut LLVMBackend, &mut AbstractTree) -> Result<IR>>,
    pub local_counter: i64,
    global_ir: Option<IR>,
    locals: Vec<Vec<Assignee>>,
}

impl LLVMBackend {
    pub fn new(a: AbstractTree) -> LLVMBackend {
        LLVMBackend {
            abstract_tree: Some(a),
            transformations: HashMap::new(),
            global_ir: Some(vec![
                "target datalayout = \"e-m:e-i64:64-f80:128-n8:16:32:64-S128\"".to_string(),
                "%object = type { i64, i64 }".to_string(),
                "declare %object @print_number() #0".to_string(),
            ]),
            local_counter: 0,
            locals: vec![],
        }
    }

    pub fn start_stack(&mut self) {
        self.local_counter = 0;
        self.locals.push(vec![]);
    }

    pub fn end_stack(&mut self) {
        self.local_counter = 0;
        self.locals.pop().unwrap();
    }


    pub fn inc_counter(&mut self) -> i64 {
        self.local_counter += 1;
        let a = self.local_counter;
        a
    }


    pub fn handle(mut self,
                  key: String,
                  f: fn(&mut LLVMBackend, &mut AbstractTree) -> Result<IR>)
                  -> LLVMBackend {
        self.transformations.insert(key, f);
        self
    }

    pub fn compile(&mut self) -> Result<IR> {
        let mut abstract_tree = self.abstract_tree.take().unwrap();
        match abstract_tree {
            Node(ref mut ats, _) => {
                ats.iter_mut().fold(Ok(vec![]), |acc, node| {
                    acc.and_then(|mut ir| {
                        self.compile_inner(node).and_then(|mut most_recently_compiled| {
                            ir.append(&mut most_recently_compiled);
                            Ok(ir)
                        })
                    })
                })
            }
            _ => {
                panic!("there should not be a node at the top level OR only call compile on the \
                        top level.")
            }
        }
        .map(|mut ir| {
            let mut global_ir = self.global_ir.take().unwrap();
            global_ir.append(&mut ir);
            global_ir
        })
    }

    pub fn compile_function_call(&mut self, tree: &mut AbstractTree) -> Result<IR> {
        match tree {
            &mut Node(ref mut ats, ref position) => {
                let length = ats.len();
                let mut iterator = ats.iter_mut();
                let mut first_item = iterator.next().unwrap();
                if length == 1 {
                    self.compile_inner(first_item)
                } else if length >= 1 {
                    match first_item {
                        &mut Node(_, ref position) => {
                            err_position(position.clone(),
                                         "unimplemented: no support for calling closures yet \
                                          implemented"
                                             .to_string())
                        }
                        &mut Token(TokenType::Symbol, ref mut fuction_name, _) => {
                            let mut i = 0;
                            let mut ir = iterator.fold(Ok(vec![]), |acc, argument| {
                                acc.and_then(|mut vector| {
                                    self.compile_inner(argument).and_then(|mut argument_ir| {
                                        argument_ir.push(format!("%object %arg{} = %{}",
                                                                 i,
                                                                 self.local_counter));
                                        i += 1;
                                        vector.append(&mut argument_ir);
                                        Ok(vector)
                                    })
                                })
                            });

                            let argument_names = self::utils::generate_function_arguments(length);
                            ir.iter_mut()
                              .map(|inner| {
                                  inner.push(format!("%{} = call %object @{}{}",
                                                     self.inc_counter(),
                                                     fuction_name,
                                                     argument_names))
                              })
                              .collect::<Vec<_>>();
                            ir
                        }
                        &mut Token(ref token_type, ref data, ref position) => {
                            err_position(position.clone(),
                                         format!("cannot call token {} of type {:?}",
                                                 data,
                                                 token_type))
                        }
                    }
                } else {
                    err_position(position.clone(), "node with zero items".to_string())
                }
            }
            _ => panic!("compile_function_call not called on a node."),
        }
    }

    pub fn compile_token(&mut self, tree: &mut AbstractTree) -> Result<IR> {
        match tree {
            &mut Token(TokenType::Symbol, ref name, _) => {
                // right now, a single symbol is a function call
                // with no argumnts. This will change with contexts.
                Ok(vec![format!("%{} = call %object @{}()", self.inc_counter(), name)])
            }
            &mut Token(TokenType::Int, ref integer_literal, _) => {
                Ok(vec![format!("%{} =l {}", self.inc_counter(), integer_literal)])
            }
            _ => tree.err("compile_token not called on a token.".to_string()),
        }
    }

    pub fn compile_inner(&mut self, tree: &mut AbstractTree) -> Result<IR> {
        {
            let transformation = self.transformations
                                     .get(&tree.name().clone())
                                     .map(|trsfmts| *trsfmts);
            if transformation.is_some() {
                let function = transformation.unwrap();
                return function(self, tree);
            }
        }
        if tree.is_node() {
            self.compile_function_call(tree)
        } else {
            self.compile_token(tree)
        }
    }
}
