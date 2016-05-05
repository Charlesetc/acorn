// parser/mod.rs

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use utils::Position;
use super::compiler::abstract_tree::{AbstractTree, TokenType};


struct Parser<'a> {
    table: HashMap<char, fn(&mut Parser) -> Option<AbstractTree>>,
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

    fn read_as(mut self, key: char, f: fn(&mut Parser) -> Option<AbstractTree>) -> Parser<'a> {
        self.table.insert(key, f);
        self
    }

    fn parse_expression(&mut self) -> Option<AbstractTree> {
        let current_reader = self.current_reader().unwrap_or(Parser::default_parse);
        current_reader(self)
    }

    fn current_reader(&mut self) -> Option<fn(&mut Parser) -> Option<AbstractTree>> {
        if self.current_char().is_none() {
            return None;
        }
        self.table.get(self.stream.peek().unwrap()).map(|f| *f)
    }

    fn current_char(&mut self) -> Option<&char> {
        self.stream.peek()
    }

    fn default_parse<'b>(parser: &mut Parser) -> Option<AbstractTree> {
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
            return None;
        }
        Some(AbstractTree::Token(TokenType::Symbol, chars, starting_position))
    }
}

fn no_op(parser: &mut Parser) -> Option<AbstractTree> {
    parser.advance_char();
    None
}

fn close_paren(parser: &mut Parser) -> Option<AbstractTree> {
    parser.advance_char();
    Some(AbstractTree::Token(TokenType::Flag, "close paren".to_string(), Position(0, 0)))
}

fn open_paren(parser: &mut Parser) -> Option<AbstractTree> {
    parser.advance_char();
    let mut accumulator = vec![];
    let starting_position = parser.position.clone();
    loop {
        let expression = parser.parse_expression();
        match expression {
            Some(AbstractTree::Token(TokenType::Flag, s, _)) => {
                if s == "close paren".to_string() {
                    break;
                }
            }
            Some(a) => accumulator.push(a),
            // TODO: Wrap all this in real error handling, I mean come on.
            None => {
                if parser.at_eof() {
                    panic!("hit end of file while reading open paren")
                } // otherwise continue onwards - just hit a space.
            }
        }
    }
    Some(AbstractTree::Node(accumulator, starting_position))
}

pub fn parse(string: &str) -> Option<AbstractTree> {
    Parser::new(string)
        .read_as(' ', no_op)
        .read_as(')', close_paren)
        .read_as('(', open_paren)
        .parse_expression()
}