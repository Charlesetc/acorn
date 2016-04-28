// compiler/mod.rs

mod visitor;

#[cfg(test)]
mod tests;

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
    fn visit_before(&mut self, f: fn(&mut AbstractTree)) {
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

    fn visit_after(&mut self, f: fn(&mut AbstractTree)) {
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

    fn match_symbol(&mut self, s: &'a str, f: fn (&mut AbstractTree)) {
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



fn my_first_visit(at: &mut AbstractTree) {
    println!("{:?}\n", at);
}

fn check_define(at: &mut AbstractTree) {
    println!("check define {:?}\n", at);
}

pub fn compile<'a>(mut at: AbstractTree<'a>) {
    at.visit_after(my_first_visit);
    at.match_symbol("define", check_define);
}
