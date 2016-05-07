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

    fn read_as(mut self,
               key: char,
               f: fn(&mut Parser) -> Result<Option<AbstractTree>>)
               -> Parser<'a> {
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
                Some(_) => {
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

    fn parse_complete(&mut self) -> Result<Option<AbstractTree>> {
        let starting_position = self.position.clone();
        let mut accumulator = vec![];
        loop {
            let expression = top_level_expression(self);
            match expression {
                Ok(Some(AbstractTree::Token(TokenType::Flag, s, p))) => {
                    // this way I can ensure ( } doesnt happen
                    return err_position(starting_position,
                                        format!("encountered incorrect flag {},\
                                                at position {:?}, while reading an \
                                                the top level",
                                                s,
                                                p));
                }
                Ok(Some(a)) => accumulator.push(a),
                Ok(None) => {
                    if self.at_eof() {
                        break;
                    }
                }
                error @ Err(_) => return error,
            }
        }
        Ok(Some(AbstractTree::Node(accumulator, starting_position)))
    }

    // Newline-specific logic - top level only
    //
    fn remove_newlines(&mut self) {
        self.table.insert('\n', no_op);
    }

    fn add_newlines(&mut self) {
        self.table.insert('\n', newline);
    }
}

fn no_op(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(None)
}


// consolidate these into one function.
fn close_paren(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(Some(AbstractTree::Token(TokenType::Flag, ")".to_string(), parser.position.clone())))
}

fn close_curly(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(Some(AbstractTree::Token(TokenType::Flag, "}".to_string(), parser.position.clone())))
}

fn newline(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    parser.advance_char();
    Ok(Some(AbstractTree::Token(TokenType::Flag, "\n".to_string(), parser.position.clone())))
}

macro_rules! define_expression_parser {
    (
        $a: ident
        name: $name: expr,
        close: $close: expr,
        advance: $should_advance: expr,
        top_level: $top_level: expr,
        ignore_newlines: $should_ignore_newlines: expr,
    ) => {

        fn $a(parser: &mut Parser) -> Result<Option<AbstractTree>> {
            let starting_position = parser.position.clone();
            if $should_advance {
                parser.advance_char();
            }
            let mut accumulator = vec![];
            loop {

                if !$should_ignore_newlines { parser.remove_newlines() }

                let expression = parser.parse_expression();

                if !$should_ignore_newlines { parser.add_newlines() }

                match expression {
                    Ok(Some(AbstractTree::Token(TokenType::Flag, s, p))) => {
                        if s == $close.to_string() {
                            break;
                        } else {
                            // this way I can ensure ( } doesnt happen
                            return err_position(starting_position,
                                                format!("encountered incorrect flag {},\
                                                        at position {:?}, while reading an \
                                                        {}", s, p, $name))
                        }
                    }
                    Ok(Some(a)) => accumulator.push(a),
                    Ok(None) => {
                        if parser.at_eof() {
                            if $top_level { break; }

                            return err_position(starting_position,
                                                format!("hit end of file \
                                                        while reading an {}", $name))
                        }
                    }
                    error @ Err(_) => return error,
                }
            }

            if accumulator.is_empty() { // This might be a bug // && parser.at_eof() {
                return Ok(None)
            }

            Ok(Some(AbstractTree::Node(accumulator, starting_position)))
        }

    }
}

define_expression_parser! { open_paren
    name: "open paren",
    close: ")",
    advance: true,
    top_level: false,
    ignore_newlines: false,
}

define_expression_parser! { top_level_expression
    name: "top level expression",
    close: "\n",
    advance: false,
    top_level: true,
    ignore_newlines: true,
}


pub fn parse(string: &str) -> Result<Option<AbstractTree>> {
    Parser::new(string)
        .read_as('\n', newline)
        .read_as(' ', no_op)
        .read_as(')', close_paren)
        .read_as('(', open_paren)
        .read_as('}', close_curly)
        // .read_as('{', open_curly)
        .parse_complete()
}

#[cfg(test)]
mod tests {
    use parser::parse;
    use compiler::abstract_tree::AbstractTree::*;
    use compiler::abstract_tree::TokenType::*;
    use utils::{Position, Error};

    macro_rules! assert_parses {
        ( $str: expr, $( $node: expr ),* ) => {
            assert_eq!(parse($str).unwrap().unwrap(),
                       Node(vec![
                            $( $node ),*
                       ], Position(0,0)))
        }
    }

    #[test]
    fn test_parse_symbol() {
        assert_parses!("symbol",
                       Node(vec![Token(Symbol, "symbol".to_string(), Position(0, 0))],
                            Position(0, 0)));
    }

    #[test]
    fn test_parse_parentheses() {
        assert_parses!("(hi there)",
                       Node(vec![Node(vec![Token(Symbol, "hi".to_string(), Position(0, 1)),
                                           Token(Symbol, "there".to_string(), Position(0, 4))],
                                      Position(0, 0))],
                            Position(0, 0)));

        // Try with two levels
        assert_parses!("(hi (one) there)",
                       Node(vec![Node(vec![Token(Symbol, "hi".to_string(), Position(0, 1)),
                                           Node(vec![Token(Symbol,
                                                           "one".to_string(),
                                                           Position(0, 5))],
                                                Position(0, 4)),
                                           Token(Symbol, "there".to_string(), Position(0, 10))],
                                      Position(0, 0))],
                            Position(0, 0)));
    }

    #[test]
    fn test_fail_parse_parentheses() {
        match parse("(hi there") {
            Ok(_) => panic!("I'm assertng this should not parse correctly"),
            Err(Error { description, position: _ }) => {
                assert_eq!("hit end of file while reading an open paren".to_string(),
                           description);
            }
        }
    }

    #[test]
    fn test_two_lines_of_code() {
        assert_parses!("hi there\n(one two)",
                       Node(vec![Token(Symbol, "hi".to_string(), Position(0, 0)),
                                 Token(Symbol, "there".to_string(), Position(0, 3))],
                            Position(0, 0)),
                       Node(vec![
                    Node(vec![
                        Token(Symbol, "one".to_string(), Position(1, 1)),
                        Token(Symbol, "two".to_string(), Position(1, 5))
                    ], Position(1,0)),
                ],
                            Position(1, 0)))
    }

    #[test]
    fn test_parse_parentheses_with_newline() {
        assert_parses!("(hi \n\n\nthere)",
                       Node(vec![Node(vec![Token(Symbol, "hi".to_string(), Position(0, 1)),
                                           Token(Symbol, "there".to_string(), Position(3, 0))],
                                      Position(0, 0))],
                            Position(0, 0)));
    }

}
