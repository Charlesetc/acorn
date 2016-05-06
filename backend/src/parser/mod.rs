// parser/mod.rs

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use utils::{Result, err_position, Position};
use super::compiler::abstract_tree::{AbstractTree, TokenType};


struct Parser<'a> {
    table: HashMap<char, fn(&mut Parser) -> Result<Option<AbstractTree>>>,
    stream: Peekable<Chars<'a>>,
    position: Position,
}

impl<'a> Parser<'a> {
    fn new(string: &'a str) -> Parser<'a> {
        Parser {
            table: HashMap::new(),
            stream: string.chars().peekable(),
            position: Position(0, 0),
        }
    }

    fn advance_char(&mut self) -> Option<char> {
        let current_char = self.stream.next();
        match current_char {
            Some(c) => {
                if c == '\n' {
                    // column set to 0
                    // line incremented
                    self.position.0 += 1;
                    self.position.1 = 0;
                } else {
                    // column incremented
                    self.position.1 += 1;
                }
            }
            None => {}
        }
        current_char
    }

    fn at_eof(&mut self) -> bool {
        self.current_char().is_none()
    }

    fn read_as(mut self, key: char, f: fn(&mut Parser) -> Result<Option<AbstractTree>>) -> Parser<'a> {
        self.table.insert(key, f);
        self
    }

    fn parse_expression(&mut self) -> Result<Option<AbstractTree>> {
        let current_reader = self.current_reader().unwrap_or(Parser::default_parse);
        current_reader(self)
    }

    fn current_reader(&mut self) -> Option<fn(&mut Parser) -> Result<Option<AbstractTree>>> {
        if self.current_char().is_none() {
            return None;
        }
        self.table.get(self.stream.peek().unwrap()).map(|f| *f)
    }

    fn current_char(&mut self) -> Option<&char> {
        self.stream.peek()
    }

    fn default_parse<'b>(parser: &mut Parser) -> Result<Option<AbstractTree>> {
        let mut chars = String::new();
        let starting_position = parser.position.clone();

        loop {
            // refactor this to a while - let
            // if parser.at_eof() { break; } // I don't think this is necessary
            let current_reader = parser.current_reader();
            match current_reader {
                Some(f) => {
                    break;
                }
                None => {
                    match parser.advance_char() {
                        Some(c) => chars.push(c),
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        if chars.len() == 0 {
            return Ok(None);
        }
        Ok(Some(AbstractTree::Token(TokenType::Symbol, chars, starting_position)))
    }

}

fn no_op(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(None)
}


// consolidate these into one function.
fn close_paren(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(Some(AbstractTree::Token(TokenType::Flag, ")".to_string(), Position(0, 0))))
}

fn close_curly(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(Some(AbstractTree::Token(TokenType::Flag, "}".to_string(), Position(0, 0))))
}

fn open_paren(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    let starting_position = parser.position.clone();
    parser.advance_char();
    let mut accumulator = vec![];
    loop {
        let expression = parser.parse_expression();
        match expression {
            Ok(Some(AbstractTree::Token(TokenType::Flag, s, p))) => {
                if s == ")".to_string() {
                    break;
                } else {
                    // this way I can ensure ( } doesnt happen
                    return err_position(starting_position,
                                        format!("encountered incorrect flag {},\
                                                at position {:?}, while reading an \
                                                open paren", s, p))
                }
            }
            Ok(Some(a)) => accumulator.push(a),
            Ok(None) => {
                if parser.at_eof() {
                    return err_position(starting_position, "hit end of file while reading an open paren".to_string())
                }
            }
            error @ Err(_) => return error,
        }
    }
    Ok(Some(AbstractTree::Node(accumulator, starting_position)))
}

pub fn parse(string: &str) -> Result<Option<AbstractTree>> {
    Parser::new(string)
        .read_as(' ', no_op)
        .read_as(')', close_paren)
        .read_as('(', open_paren)
        .read_as('}', close_curly)
        // .read_as('{', open_curly)
        .parse_expression()
}

#[cfg(test)]
mod tests {
    use parser::parse;
    use compiler::abstract_tree::AbstractTree::*;
    use compiler::abstract_tree::TokenType::*;
    use utils::{err_position, Position, Error};

    #[test]
    fn test_parse_symbol() {
        assert_eq!(
            parse("symbol").unwrap().unwrap(),
            Token(Symbol, "symbol".to_string(), Position(0,0))
        )
    }

    #[test]
    fn test_parse_parentheses() {
        assert_eq!(
            parse("(hi there)").unwrap().unwrap(),
            Node(vec![Token(Symbol, "hi".to_string(), Position(0,1)), Token(Symbol, "there".to_string(), Position(0,4))], Position(0,0))
        );

        // Try with two levels
        assert_eq!(
            parse("(hi (one) there)").unwrap().unwrap(),
            Node(vec![Token(Symbol, "hi".to_string(), Position(0,1)), Node(vec![Token(Symbol, "one".to_string(), Position(0,5))], Position(0,4)), Token(Symbol, "there".to_string(), Position(0, 10))], Position(0,0))
        );
    }

    #[test]
    fn test_fail_parse_parentheses() {
        match parse("(hi there") {
            Ok(_) => panic!("I'm assertng this should not parse correctly"),
            Err(Error { description: description, position: _ }) => {
                assert_eq!("hit end of file while reading an open paren".to_string(), description);
            }
        }
    }
}
