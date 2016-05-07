// parser/mod.rs

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use utils::{Result, err_position, Position};
use super::compiler::abstract_tree::{AbstractTree, TokenType, BLOCK_IDENTIFIER};

fn node_token() -> AbstractTree {
    AbstractTree::Token(TokenType::Symbol,
                        BLOCK_IDENTIFIER.to_string(),
                        Position(0, 0))
}

struct Parser<'a> {
    table: HashMap<char, fn(&mut Parser) -> Result<Option<AbstractTree>>>,
    stream: Peekable<Chars<'a>>,
    position: Position,
    top_level: bool,

    last_char: Option<char>, // this is helpful for parsing blocks
}

impl<'a> Parser<'a> {
    fn new(string: &'a str) -> Parser<'a> {
        Parser {
            table: HashMap::new(),
            stream: string.chars().peekable(),
            position: Position(0, 0),
            top_level: true,

            last_char: None,
        }
    }

    fn advance_char(&mut self) -> Option<char> {
        let current_char = self.stream.next();
        self.last_char = current_char.clone();

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

                if $should_ignore_newlines { parser.remove_newlines() }

                let expression = parser.parse_expression();

                if $should_ignore_newlines { parser.add_newlines() }

                match expression {
                    Ok(Some(AbstractTree::Token(TokenType::Flag, s, p))) => {
                        if $close.contains(&s) {
                            break;
                        } else {
                            // this way I can ensure ( } doesnt happen
                            return err_position(starting_position,
                                                format!("encountered incorrect flag '{}',\
                                                        at position {:?}, while reading \
                                                        {}", s, p, $name))
                        }
                    }
                    Ok(Some(a)) => accumulator.push(a),
                    Ok(None) => {
                        if parser.at_eof() {
                            if $top_level { break; }

                            return err_position(starting_position,
                                                format!("hit end of file \
                                                        while reading {}", $name))
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

macro_rules! define_aggregate_parser {

    ( $name: ident
      parser: $inner_parser: expr, ) => {

        fn $name(parser: &mut Parser) -> Result<Option<AbstractTree>> {
            let starting_position = parser.position.clone();
            let mut accumulator = vec![];
            loop {
                let expression = try!($inner_parser(parser));
                match expression {
                    Some(AbstractTree::Token(_, s, p)) => {
                        panic!("tokens shouldn't be returned by
                               the inner parser of an aggregate parser");
                    }
                    Some(a) => {
                        accumulator.push(a)
                    }
                    None => {
                        if parser.at_eof() {
                            break;
                        }
                    }
                }
            }
            Ok(Some(AbstractTree::Node(accumulator, starting_position)))
        }

    }
}

define_expression_parser! { open_paren
    name: "an open paren",
    close: ")",
    advance: true,
    top_level: false,
    ignore_newlines: true,
}

// this is used for parsting top level expressions
// i.e. expressions without any nesting or parens.
define_expression_parser! { parse_whole_expression
    name: "top level expressions",
    close: "\n",
    advance: false,
    top_level: true,
    ignore_newlines: false,
}

// this is used for the first expression within a block - it parses differently
// depending on whether or not it terminates with a '}' or a '\n'
define_expression_parser! { parse_whole_expression_block_starting
    name: "expressions of a block",
    close: "}\n", // either one of these will work
    advance: false,
    top_level: true,
    ignore_newlines: false,
}

// this is used for parsing expressions within a block, i.e.
// anything within the { } after the first expression.
define_expression_parser! { parse_whole_expression_block
    name: "expressions of a block",
    close: "}",
    advance: false,
    top_level: true,
    ignore_newlines: false,
}

define_aggregate_parser! { complete_parse
    parser: parse_whole_expression,
}

define_aggregate_parser! { complete_parse_block
    parser: parse_whole_expression_block_starting,
}


fn open_curly(parser: &mut Parser) -> Result<Option<AbstractTree>> {
    let starting_position = parser.position.clone();
    parser.advance_char();
    let mut expression = try!(parse_whole_expression_block_starting(parser));
    loop {
        match expression {
            Some(AbstractTree::Token(_, s, p)) => {
                panic!("inner parser to an aggregater should not return tokens")
            }
            Some(_) => {
                break;
            } // this can be unwrapped safely in the future.
            None => {
                if parser.at_eof() {
                    return err_position(starting_position,
                                        format!("hit end of file \
                                                while reading a block"));
                }

                // Make this logic better.
                if parser.last_char == Some('\n') {
                    // position will not be used.
                    expression = Some(AbstractTree::Node(vec![], Position(0, 0)));
                    break;
                } else {
                    expression = try!(parse_whole_expression_block_starting(parser));
                }
            }
        }
    }
    if parser.last_char.is_none() {
        return err_position(starting_position,
                            format!("hit end of file while reading a block"));
    } else if parser.last_char == Some('\n') {
        let mut expression = expression.unwrap();
        let position = expression.position();


        let mut arguments = match expression {
            AbstractTree::Node(vector, _) => vector,
            _ => panic!("fetching arguments on not a node"),
        };

        arguments.insert(0, node_token());

        // you might not need this iteration
        let mut block = try!(complete_parse_block(parser));
        loop {
            match block {
                Some(AbstractTree::Token(_, s, p)) => {
                    panic!("inner parser to an aggregater should not return tokens")
                }
                Some(a) => {
                    arguments.push(a);
                    break;
                }
                None => {
                    if parser.at_eof() {
                        return err_position(starting_position,
                                            format!("hit end of file \
                                                    while reading a block"));
                    }
                    block = try!(complete_parse_block(parser));
                }
            }
        }

        Ok(Some(AbstractTree::Node(arguments, position)))
    } else {
        assert_eq!(parser.last_char, Some('}'));

        let expression = expression.unwrap();
        let arguments = vec![node_token(), expression];

        Ok(Some(AbstractTree::Node(arguments, starting_position)))
    }
}



pub fn parse(string: &str) -> Result<Option<AbstractTree>> {
    let mut parser = Parser::new(string)
                         .read_as('\n', newline)
                         .read_as(' ', no_op)
                         .read_as(')', close_paren)
                         .read_as('(', open_paren)
                         .read_as('}', close_curly)
                         .read_as('{', open_curly);
    complete_parse(&mut parser)
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

    #[test]
    fn test_parses_block_simple() {
        assert_parses!("{ one two }",
                       Node(vec![Node(vec![Token(Symbol, "block".to_string(), Position(0, 0)),
                                           Node(vec![Token(Symbol,
                                                           "one".to_string(),
                                                           Position(0, 2)),
                                                     Token(Symbol,
                                                           "two".to_string(),
                                                           Position(0, 6))],
                                                Position(0, 1))],
                                      Position(0, 0))],
                            Position(0, 0)))
    }

    #[test]
    fn test_parses_block_newline() {
        assert_parses!("{\none two\n}",
                       Node(vec![Node(vec![Token(Symbol, "block".to_string(), Position(0, 0)),
                                           Node(vec![
                                        Node(vec![Token(Symbol, "one".to_string(), Position(1, 0)),
                                                Token(Symbol, "two".to_string(), Position(1, 4))
                                        ], Position(1, 0)),
                                      ],
                                                Position(1, 0))],
                                      Position(0, 0))],
                            Position(0, 0)))
    }


    #[test]
    fn test_parses_block_complete() {
        assert_parses!("map { a\ntimes a 2\nreturn 4\n\n}",
                       Node(vec![Token(Symbol, "map".to_string(), Position(0, 0)),
                           Node(vec![Token(Symbol, "block".to_string(), Position(0, 0)),
                                        Token(Symbol, "a".to_string(), Position(0, 6)),
                                           Node(vec![
                                                Node(vec![Token(Symbol, "times".to_string(), Position(1, 0)),
                                                          Token(Symbol, "a".to_string(), Position(1, 6)),
                                                          Token(Symbol, "2".to_string(), Position(1, 8)),
                                                ], Position(1, 0)),
                                                Node(vec![Token(Symbol, "return".to_string(), Position(2, 0)),
                                                          Token(Symbol, "4".to_string(), Position(2, 7)),
                                                ], Position(2, 0)),
                                          ], Position(1, 0))],
                                      Position(0, 5))],
                            Position(0, 0)))
    }

    // TODO: Add this test for the future.
    // #[test]
    // fn test_parses_blocks_on_lines() {
    //     assert_parses!("map { something } two\n foo bar",)
    // }

}
