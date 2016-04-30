// compiler/abstract_tree.rs

#[derive(Debug)]
pub enum TokenType {
    Symbol,
    Str,
    Int,
    Float,
}

#[derive(Debug)]
pub enum AbstractTree<'a> {
    Node(Vec<AbstractTree<'a>>),
    Token(TokenType, &'a str),
}

impl<'a> AbstractTree<'a> {
    pub fn visit_before(&mut self, f: fn(&mut AbstractTree)) {
        match self {
            &mut AbstractTree::Node(ref mut ats) => {
                ats.iter_mut().map(|mut x| {
                    x.visit_before(f);
                    f(x);
                }).collect::<Vec<_>>();
            }
            _ => { }
        }
    }

    pub fn visit_after(&mut self, f: fn(&mut AbstractTree)) {
        match self {
            &mut AbstractTree::Node(ref mut ats) => {
                ats.iter_mut().map(|mut x| {
                    f(x);
                    x.visit_after(f);
                }).collect::<Vec<_>>();
            }
            _ => { }
        }
    }

    pub fn match_symbol(&mut self, s: &'a str, f: fn (&mut AbstractTree)) {
        let matched = match self {
            &mut AbstractTree::Node(ref mut ats) => {
                match ats.get_mut(0) {
                    Some(&mut AbstractTree::Token(TokenType::Symbol, a)) => s == a,
                    _ => false,
                }
            },
            _ => false,
        };

        if matched {
            f(self);
        }

        match self {
            &mut AbstractTree::Node(ref mut ats) => {
                ats.iter_mut().map(|mut x| {
                    x.match_symbol(s, f);
                }).collect::<Vec<_>>();
            }
            _ => { return }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::AbstractTree;
    use super::AbstractTree::*;
    use super::TokenType::*;

    fn generate_data<'a>() -> AbstractTree<'a> {
        return Node(vec![
            Node(vec![
                 Token(Symbol, "foo"),
                 Token(Int, "2"),
                 Token(Int, "2"),
                 Node(vec![
                        Token(Symbol, "foo"),
                        Token(Symbol, "foo"),
                ]),
            ]),
            Node(vec![
                 Token(Symbol, "define"),
                 Token(Int, "a"),
                 Token(Int, "2")
            ]),
        ]);
    }

    static mut foo_visitor_count: i64 = 0;
    fn foo_visitor(at: &mut AbstractTree) { unsafe { foo_visitor_count += 1; } }
    #[test]
    fn test_match_symbol() {
        let mut data = generate_data();
        data.match_symbol("foo", foo_visitor);
        assert_eq!(unsafe { foo_visitor_count }, 2);
        unsafe { foo_visitor_count = 0 };

        data.visit_before(foo_visitor);
        assert_eq!(unsafe { foo_visitor_count }, 11);
    }

}
