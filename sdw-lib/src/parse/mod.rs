use crate::prelude::*;

#[derive(Debug)]
pub enum Type {
    Int,
    Void,
}

impl Type {
    fn from_string(other: String) -> Type {
        match other.as_str() {
            "int" => Type::Int,
            "void" => Type::Void,
            _ => panic!("TODO(5DA): oopsie doopsie"),
        }
    }
}

#[derive(Debug)]
pub struct Parameter(String, Type);
// TODO(5DA): stub
#[derive(Debug)]
pub struct Expression(i64);

#[derive(Debug)]
pub enum Node {
    Function {
        params: Vec<Parameter>,
        name: String,
        return_ty: Type,
    },
    Return {
        expr: Expression,
    },
}

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
#[derive(Debug)]
pub struct Parser {
    lexemes: LexemeStream,
    pub ast: Vec<Node>,
}

impl Parser {
    pub fn new(lexemes: Vec<Lexeme>) -> Self {
        Self {
            lexemes: lexemes.into(), ast: Vec::new()
        }
    }

    // TODO: errors
    pub fn consume(&mut self, ty: LexemeTypes) -> Lexeme {
        let tk = self.lexemes.pop_front().unwrap();
        if tk.ty == ty {
            tk
        } else {
            panic!("TODO(5DA): wtf is this lol");
        }
    }

    pub fn eat_idn(&mut self) -> String {
        match self.lexemes.pop_front().unwrap().ty {
            LexemeTypes::Idn(s) => s,
            _ => panic!("TODO(5DA): ditto lol"),
        }
    }

    pub fn eat_expr(&mut self) -> Expression {
        // TODO(5DA): pratt parsing
        match self.lexemes.pop_front().unwrap().ty {
            LexemeTypes::Literal(Literal::Integer(i)) => Expression(i),
            _ => panic!("TODO(5DA): it just gets worse and worse, hm"),
        }
    }

    // TODO(5DA): consider newtyping `Option<()>` here.
    pub fn parse(&mut self) -> Result<Option<()>> {
        if self.lexemes.is_empty() {
            return Ok(None);
        }

        // TODO(5DA): we're gonna wanna newtype this at some point..
        let next = self.lexemes.pop_front().unwrap();
        match next.ty {
            LexemeTypes::Keyword(kw) => match kw {
                Keywords::Fn => {
                    let ty = self.eat_idn();
                    let nm = self.eat_idn();
                    self.consume(LexemeTypes::OpenParen);
                    self.consume(LexemeTypes::CloseParen);
                    println!("function with name {nm} and type {ty}");
                    self.ast.push(Node::Function {
                        params: Vec::new(),
                        name: nm,
                        return_ty: Type::from_string(ty),
                    });
                },
                Keywords::Return => {
                    let v = self.eat_expr();
                    println!("return statement with value {}", v.0);
                    self.ast.push(Node::Return {
                        expr: v,
                    });
                },
            },
            _ => {}
        }
        self.parse()?;
        Ok(Some(()))
    }
}



