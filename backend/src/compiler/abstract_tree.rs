// compiler/abstract_tree.rs

use utils::{
    Result,
    Error,
    Position
};
use self::AbstractTree::*;

static BLOCK_IDENTIFIER: &'static str = "block";

#[derive(Debug)]
pub enum TokenType {
    Symbol,
    Str,
    Int,
    Float,
}

#[derive(Debug)]
pub enum AbstractTree<'a> {
    Node(Vec<AbstractTree<'a>>, Position),
    Token (
        TokenType,
        &'a str,
        Position
    ),
}

impl<'a> AbstractTree<'a> {
    // pub fn visit_before(&mut self, f: fn(&mut AbstractTree)) {
    //     match self {
    //         &mut Node(ref mut ats) => {
    //             ats.iter_mut().map(|mut x| {
    //                 x.visit_before(f);
    //                 f(x);
    //             }).collect::<Vec<_>>();
    //         }
    //         _ => { }
    //     }
    // }

    pub fn match_symbol(&mut self, s: &'a str, f: fn (&mut AbstractTree) -> Result<()>) -> Result<()> {
        let start = match self {
            &mut Node(ref ats, _) => {
                match ats.get(0) {
                    Some(&Token(TokenType::Symbol, a, _)) => {
                        if s == a {
                            // this is so I can destructure immutably
                            // and then call f on the mutable self object
                            self.err("this is a goto to f(self)".to_string())
                        } else {
                            Ok(())
                        }
                    },
                    _ => Ok(()),
                }
            },
            _ => Ok(()),
        }.or_else(|_| f(self) );

        match self {
            &mut Node(ref mut ats, _) => {
                ats.iter_mut().fold(start, |results, x| {
                    results.and_then(|_| x.match_symbol(s, f))
                })
            }
            _ => { start }
        }
    }


    // Functions for validating ast

    pub fn check_length(&self, i: usize) -> Result<()> {
        match self {
            &Node(ref ats, _) => {
                if ats.len() == i {
                    Ok(())
                } else {
                    self.err(format!("{} takes {} arguments", self.name(), i-1))
                }
            }
            _ => { panic!(format!{"check_length called on not a node: {:?}", self}) }
        }
    }

    pub fn check_argument_block(&self, argument_number: usize) -> Result<()> {
        let error = self.err(format!("{} expects a block for its {}th argument",
                                     self.name(),
                                     argument_number));
        let argument = self.argument(argument_number);
        Ok(())
            .and_then(|_| argument.assert_node(error.clone()) )
            .and_then(|_| argument.check_length(3) )
            .and_then(|_| {
                // make sure the block starts with 'block'
                match argument {
                    &Node(ref ats, _) => {
                        let block_error = self.err("a block takes a \
                                                     list of arguments \
                                                     followed by a list \
                                                     of expressions".to_string());
                        Ok(())
                            .and_then(|_| {
                                match ats.get(0).unwrap() {
                                    &Token(TokenType::Symbol, a, _) => {
                                        if a == BLOCK_IDENTIFIER {
                                            Ok(())
                                        } else {
                                            error.clone()
                                        }
                                    }
                                    _ => error.clone(),
                                }
                            })
                            .and_then(|_| ats[1].assert_node(block_error.clone()))
                            .and_then(|_| ats[2].assert_node(block_error.clone()))
                    }
                    _ => panic!("I already asserted this was a node."),
                }
            })
    }

    fn assert_node(&self, error: Result<()>) -> Result<()> {
        match self {
            &Node(_, _) => Ok(()),
            _ => error,
        }
    }

    // Functions for reading the ast

    pub fn argument(&self, i: usize) -> &AbstractTree<'a> {
         self.arguments().get(i).unwrap()
    }

    pub fn arguments(&self) -> &Vec<AbstractTree<'a>> {
        match self {
            &Node(ref ats, _) => { return ats },
            _ => { panic!("fn arguments called on a Token") }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            &Node(ref ats, _) => {
                match ats.get(0) {
                    Some(&Token(TokenType::Symbol, a, _)) => {
                        a
                    }
                    _ => { "unknown" }
                }
            }
            _ => { "token" }
        }
    }

    fn position(&self) -> Position {
        match self {
            &Node(_, ref position) => {
                position.clone()
            }
            &Token(_, _, ref position) => {
                position.clone()
            }
        }
    }

    fn err(&self, description: String) -> Result<()> {
        Err(Error { description: description, position: self.position() })
    }
}

#[cfg(test)]
mod tests {
    use super::AbstractTree;
    use utils::{
        Result,
    };
    use utils::tests::{
        generate_data,
    };

    static mut foo_visitor_count: i64 = 0;
    fn visitor_match_symbol(_: &mut AbstractTree) -> Result<()> { unsafe { foo_visitor_count += 1; }; Ok(()) }

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
        let error = data.err("this is an error".to_string()).err().unwrap();
        assert_eq!(error.description, "this is an error".to_string());
        assert_eq!(error.position, data.position());
    }
}
