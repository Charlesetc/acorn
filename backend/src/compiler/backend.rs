// compiler/backend.rs

use std::collections::HashMap;

use utils::{Result, IR, err_position};
use compiler::abstract_tree::{AbstractTree, TokenType};
use compiler::abstract_tree::AbstractTree::*;

mod utils {
    pub fn generate_function_arguments(start: usize, i: usize) -> String {
        let mut output = "(".to_string();
        for x in start..(start + i - 1) {
            // comma formatting (facepalm)
            if x != start {
                output.push(',');
            }
            output.push_str(&format!("%object %ret.{}", x));
        }
        output.push_str(")");
        output
    }
}

/// A struct holding information
/// about a variable that has been
/// assigned to. It has a name for the
/// variable and a location
#[derive(Debug, Clone)]
pub struct Assignee {
    pub name: String,
    /// This is incremented each time this var
    /// is assigned to. It's needed to convert to
    /// ssa.
    count: i64,
}

// probably going to get rid of this.
impl Assignee {
    fn new(name: String) -> Assignee {
        Assignee {
            name: name,
            count: -1,
        }
    }
}

pub struct LLVMBackend {
    pub abstract_tree: Option<AbstractTree>,
    pub transformations: HashMap<String, fn(&mut LLVMBackend, &mut AbstractTree) -> Result<IR>>,
    all_counter: HashMap<String, i64>,
    global_ir: Option<IR>,
    locals: Vec<HashMap<String, Assignee>>,
}

impl LLVMBackend {
    pub fn new(a: AbstractTree) -> LLVMBackend {
        LLVMBackend {
            abstract_tree: Some(a),
            transformations: HashMap::new(),
            global_ir: Some(vec![
                "target datalayout = \"e-m:e-i64:64-f80:128-n8:16:32:64-S128\"".to_string(),
                "%object = type { i64, i64 }".to_string(),
                "declare %object @print_number(%object) #0".to_string(),
            ]),
            all_counter: HashMap::new(),
            locals: vec![],
        }
    }

    pub fn start_stack(&mut self) {
        self.all_counter.remove("ret"); // okay?
        self.locals.push(HashMap::new());
    }

    pub fn end_stack(&mut self) -> HashMap<String, Assignee> {
        self.locals.pop().unwrap()
    }

    pub fn get_var_index(&mut self, key: &str) -> i64 {
        self.all_counter.get(key).map(|x| *x).unwrap_or(-1) + 1
    }

    pub fn load_var_ir(&mut self, existing_name: String, local_name: String) -> IR {
        // let assignee = self.get_assignee(&local_name)
        //     .expect("there should not be uninitialized locals");
        vec![format!("%{} = load %object, %object* %{}", existing_name, local_name)]
    }

    pub fn set_var_ir(&mut self, existing_name: &String, new_value: String) -> IR {
        if !self.get_assignee(existing_name).is_some() {
            self.add_assignee(existing_name);
        }
        vec![format!("store %object %{}, %object* %{}", new_value, existing_name)]
    }

    // this does not take into account whether
    // the variable already exists in the stack.
    pub fn add_assignee(&mut self, name: &String) {
        let assignee = Assignee::new(name.clone());
        self.locals.last_mut().unwrap().insert(name.clone(), assignee);
    }

    pub fn get_assignee(&mut self, name: &String) -> Option<Assignee> {
        // This could be improved with a hashtable if it becomes a problem.
        for stack in &self.locals {
            let assignee = stack.get(name);
            if assignee.is_some() && &assignee.unwrap().name == name {
                return Some(assignee.unwrap().clone()) // TODO: this is gross.
            }
        }
        None
    }


    pub fn get_counter(&mut self, key: &str) -> String {
        format!("{}.{}", key, self.all_counter.get(key).unwrap_or(&0).clone())
    }

    pub fn inc_counter(&mut self, key: &str) -> String {
        let i = self.all_counter.get(key).map(|i| *i);
        let v = if i.is_some() {
            let value = i.unwrap() +  1;
            self.all_counter.insert(key.to_string(), value);
            value
        } else {
            self.all_counter.insert(key.to_string(), 0);
            0
        };
        format!("{}.{}", key, v)
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

    // TODO: Move this to it's own module
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
                            let starting_index = self.get_var_index("ret") as usize;

                            let mut ir = iterator.fold(Ok(vec![]), |acc, argument| {
                                acc.and_then(|mut vector| {
                                    self.compile_inner(argument).and_then(|mut argument_ir| {
                                        vector.append(&mut argument_ir);
                                        Ok(vector)
                                    })
                                })
                            });

                            let argument_names = self::utils::generate_function_arguments(starting_index, length);
                            ir.iter_mut()
                              .map(|inner| {
                                  inner.push(format!("%{} = call %object @{}{}",
                                                     self.inc_counter("ret"),
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
                match self.get_assignee(name) {
                    Some(local) => {
                        let counter = self.inc_counter("ret");
                        Ok(self.load_var_ir(counter, local.name))
                    },
                    None => {
                        Ok(vec![format!("%{} = call %object @{}()", self.inc_counter("ret"), name)])
                    }
                }
            }
            &mut Token(TokenType::Int, ref integer_literal, _) => {
                // what the hell is happening here.
                // this is wrong.
                Ok(vec![format!("%{} =l {}", self.inc_counter("ret"), integer_literal)])
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
