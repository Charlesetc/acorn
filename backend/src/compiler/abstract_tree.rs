// compiler/abstract_tree.rs

use std::collections::HashMap;

use utils::{Result, Position, IR, err_position};
use self::AbstractTree::*;

static BLOCK_IDENTIFIER: &'static str = "block";

mod utils {
    pub fn generate_function_arguments(i: usize) -> String {
        let mut output = "(".to_string();
        for x in 0..i {
            output.push_str(&format!("l arg{}, ", x));
        }
        output.push_str(")");
        output
    }
}

/// TokenType is supposed to relay any information
/// about the Token that would be known from the first
/// pass of parsing. For example, the difference between
/// an int literal `3` or a float literal `4.3` is determined by the
/// string representation.
///
/// A `Symbol` type is the most basic - representing an ident of the language.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Flag, // Used internally - should not be encountered by outside people.
    Symbol,
    Int, /* Str,
          * Float, */
}

pub struct QBEBackend {
    pub abstract_tree: Option<AbstractTree>,
    pub transformations: HashMap<String, fn(&mut QBEBackend, &mut AbstractTree) -> Result<IR>>,
}

impl QBEBackend {
    pub fn new(a: AbstractTree) -> QBEBackend {
        QBEBackend {
            abstract_tree: Some(a),
            transformations: HashMap::new(),
        }
    }

    pub fn handle(mut self,
                  key: String,
                  f: fn(&mut QBEBackend, &mut AbstractTree) -> Result<IR>)
                  -> QBEBackend {
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
                                        i += 1;
                                        argument_ir.push(format!("%arg{} =l %ret", i));
                                        vector.append(&mut argument_ir);
                                        Ok(vector)
                                    })
                                })
                            });

                            let argument_names = self::utils::generate_function_arguments(length);
                            ir.iter_mut()
                              .map(|inner| {
                                  inner.push(format!("%ret =l call ${} {}",
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
            &mut Token(TokenType::Symbol, _, ref position) => {
                err_position(position.clone(),
                             "variables are not yet implemented".to_string())
            }
            &mut Token(TokenType::Int, ref integer_literal, _) => {
                Ok(vec![format!("%ret =l {}", integer_literal)])
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

/// The AbstractTree is what is given to the `compile`
/// function of the compiler module. It consists of
/// nodes and tokens - nodes simply hold more abstract
/// trees, whereas tokens have a TokenType and a string
/// representation. All AbstractTree's have a position
/// that is used for reporting errors.
#[derive(Debug, Eq, PartialEq)]
pub enum AbstractTree {
    Node(Vec<AbstractTree>, Position),
    Token(TokenType, String, Position),
}

impl AbstractTree {
    /// This is only called in assert_only_top_level() so it can
    /// be baseed to match_symbol()
    fn fail_for_top_leval_call(a: &mut AbstractTree) -> Result<()> {
        a.err(format!("{} was invoked without being on the top level", a.name()))
    }

    /// assert_only_top_level() will return a Result::Err if
    /// a call occurs somewhere that's not the top level.
    pub fn assert_only_top_level<'a>(&mut self, s: &'a str) -> Result<()> {
        // Go two nodes deep and assert there are no more after that.
        match self {
            &mut Node(ref mut ats, _) => {
                ats.iter_mut().fold(Ok(()), |acc, at| {
                    acc.and_then(|_| {
                        match at {
                            &mut Node(ref mut ats, _) => {
                                ats.iter_mut().fold(Ok(()), |acc, a| {
                                    acc.and_then(|_| {
                                        a.match_symbol(s, AbstractTree::fail_for_top_leval_call)
                                    })
                                })
                            }
                            _ => Ok(()),
                        }
                    })
                })
            }
            _ => Ok(()),
        }
    }

    pub fn match_symbol<'a>(&mut self,
                            s: &'a str,
                            f: fn(&mut AbstractTree) -> Result<()>)
                            -> Result<()> {
        let start = match self {
                        &mut Node(ref ats, _) => {
                            match ats.get(0) {
                                Some(&Token(TokenType::Symbol, ref a, _)) => {
                                    if s == a {
                                        // this is so I can destructure immutably
                                        // and then call f on the mutable self object
                                        self.err("this is a goto to f(self)".to_string())
                                    } else {
                                        Ok(())
                                    }
                                }
                                _ => Ok(()),
                            }
                        }
                        _ => Ok(()),
                    }
                    .or_else(|_| f(self));

        match self {
            &mut Node(ref mut ats, _) => {
                ats.iter_mut().fold(start,
                                    |results, x| results.and_then(|_| x.match_symbol(s, f)))
            }
            _ => start,
        }
    }


    // Functions for validating ast

    pub fn check_min_length(&self, i: usize) -> Result<()> {
        match self {
            &Node(ref ats, _) => {
                if ats.len() >= i {
                    Ok(())
                } else {
                    self.err(format!("{} takes at least {} arguments", self.name(), i - 1))
                }
            }
            _ => panic!(format!("check_length called on not a node: {:?}", self)),
        }
    }

    pub fn check_length(&self, i: usize) -> Result<()> {
        match self {
            &Node(ref ats, _) => {
                if ats.len() == i {
                    Ok(())
                } else {
                    self.err(format!("{} takes {} arguments", self.name(), i - 1))
                }
            }
            _ => panic!(format!("check_length called on not a node: {:?}", self)),
        }
    }

    /// check_argument_block will check to make sure the argument_number'th
    /// argument is formatted like a block.
    pub fn check_argument_block(&self, argument_number: usize) -> Result<()> {
        let error = self.err(format!("{} expects a block for its {}th argument",
                                     self.name(),
                                     argument_number));
        let argument = self.argument(argument_number);
        Ok(())
            .and_then(|_| argument.assert_node(error.clone()))
            .and_then(|_| argument.check_min_length(2))
            .and_then(|_| {
                // make sure the block starts with 'block'
                match argument {
                    &Node(ref ats, _) => {
                        let block_error = self.err("a block takes a \
                                                     list of arguments \
                                                     followed by a list \
                                                     of expressions"
                                                       .to_string());
                        Ok(())
                            .and_then(|_| {
                                match ats.get(0).unwrap() {
                                    &Token(TokenType::Symbol, ref a, _) => {
                                        if a == BLOCK_IDENTIFIER {
                                            Ok(())
                                        } else {
                                            error.clone()
                                        }
                                    }
                                    _ => error.clone(),
                                }
                            })
                            .and_then(|_| ats.last().unwrap().assert_node(block_error.clone()))
                    }
                    _ => panic!("I already asserted this was a node."),
                }
            })
    }

    /// Given an AbstractTree, assert it's a Node not a Token.
    ///
    /// # Examples:
    ///
    /// ```
    /// let error = Node(vec![], Position(0,0)).err("this is an error")
    ///
    /// Node(vec![], Position(0,0)).assert_node().ok().unwrap()
    /// Token(Symbol, "", Position(0,0)).assert_node().err().unwrap()
    /// ```
    ///
    fn assert_node(&self, error: Result<()>) -> Result<()> {
        match self {
            &Node(_, _) => Ok(()),
            _ => error,
        }
    }

    // Functions for reading the ast

    /// Get an immutable reference to the ith argument of a node.
    pub fn argument(&self, i: usize) -> &AbstractTree {
        self.arguments().get(i).unwrap()
    }

    /// Get an immutabe reference to the arguments of a node
    ///
    /// This will panic if called on a Token.
    pub fn arguments(&self) -> &Vec<AbstractTree> {
        match self {
            &Node(ref ats, _) => return ats,
            _ => panic!("fn arguments called on a Token"),
        }
    }

    pub fn arguments_mut(&mut self) -> &mut Vec<AbstractTree> {
        match self {
            &mut Node(ref mut ats, _) => return ats,
            _ => panic!("fn arguments called on a Token"),
        }
    }

    /// Get the 'name' of a Node - defined to be the
    /// string of the first token if the abstract tree is
    /// a node and has a first token.
    pub fn name<'a>(&'a self) -> &'a String {
        match self {
            &Node(ref ats, _) => {
                match ats.get(0) {
                    Some(&Token(TokenType::Symbol, ref a, _)) => a,
                    _ => panic!("cannot access the name of just any node"),
                }
            }
            &Token(_, ref data, _) => data,
        }
    }

    /// The Position of an abstract tree -
    /// both a Node and a Token have it, but
    /// accessing it requires deconstructing
    /// which is why this method is useful.
    fn position(&self) -> Position {
        match self {
            &Node(_, ref position) => position.clone(),
            &Token(_, _, ref position) => position.clone(),
        }
    }

    /// Generate an utils::Result type from a discription
    /// passed in and this abstract tree's position.
    fn err<T>(&self, description: String) -> Result<T> {
        err_position(self.position(), description)
    }

    fn is_node(&self) -> bool {
        match self {
            &Node(_, _) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AbstractTree;
    use utils::Result;
    use utils::tests::generate_data;

    static mut foo_visitor_count: i64 = 0;
    fn visitor_match_symbol(_: &mut AbstractTree) -> Result<()> {
        unsafe {
            foo_visitor_count += 1;
        };
        Ok(())
    }

    #[test]
    fn test_match_symbol() {
        let mut data = generate_data();
        data.match_symbol("foo", visitor_match_symbol).ok().unwrap();
        assert_eq!(unsafe { foo_visitor_count }, 2);
        unsafe { foo_visitor_count = 0 };
    }

    fn visitor_check_length_2(at: &mut AbstractTree) -> Result<()> {
        at.check_length(2)
    }
    fn visitor_check_length_1(at: &mut AbstractTree) -> Result<()> {
        at.check_length(1)
    }

    #[test]
    fn test_check_length() {
        let mut data = generate_data();
        assert!(data.match_symbol("define", visitor_check_length_2).is_ok());
        assert!(data.match_symbol("define", visitor_check_length_1).is_err());
    }

    #[test]
    fn test_err_and_position() {
        let data = generate_data();
        let error = data.err::<Result<()>>("this is an error".to_string()).err().unwrap();
        assert_eq!(error.description, "this is an error".to_string());
        assert_eq!(error.position, data.position());
    }
}
