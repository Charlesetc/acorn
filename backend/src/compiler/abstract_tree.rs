// compiler/abstract_tree.rs

use utils::{
    Result,
    Error,
    Position
};
use self::AbstractTree::*;

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


    // Functions for reading the ast

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
    use super::AbstractTree::*;
    use super::TokenType::*;
    use utils::Result;

    fn generate_data<'a>() -> AbstractTree<'a> {
        return Node(vec![
            Node(vec![
                 Token(Symbol, "foo", Position(0,0)),
                 Token(Int, "2", Position(0,0)),
                 Token(Int, "2", Position(0,0)),
                 Node(vec![
                        Token(Symbol, "foo", Position(0,0)),
                        Token(Symbol, "foo", Position(0,0)),
                ], Position(0, 2)),
            ], Position(0, 2)),
            Node(vec![
                 Token(Symbol, "define", Position(0,0)),
                 Token(Int, "2", Position(0,0))
            ], Position(0, 2)),
        ], Position(0, 2));
    }

    static mut foo_visitor_count: i64 = 0;
    fn visitor_match_symbol(at: &mut AbstractTree) -> Result<()> { unsafe { foo_visitor_count += 1; }; Ok(()) }

    #[test]
    fn test_match_symbol() {
        let mut data = generate_data();
        data.match_symbol("foo", visitor_match_symbol);
        assert_eq!(unsafe { foo_visitor_count }, 2);
        unsafe { foo_visitor_count = 0 };
    }

    fn visitor_check_length_3(at: &mut AbstractTree) -> Result<()> {
        at.check_length(3)
    }
    fn visitor_check_length_1(at: &mut AbstractTree) -> Result<()> {
        at.check_length(1)
    }

    #[test]
    fn test_check_length() {
        let mut data = generate_data();
        assert!(data.match_symbol("define", visitor_check_length_3).is_ok());
        assert!(data.match_symbol("define", visitor_check_length_1).is_err());

    }

}
