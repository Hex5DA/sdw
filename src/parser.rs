use crate::prelude::*;

#[derive(Debug)]
pub enum Bound {}
#[derive(Debug)]
pub enum Expr {}
#[derive(Debug)]
pub enum Stmt {}

pub type STN = Spanned<ST>;
#[derive(Debug)]
pub enum ST {
    Idn(String),
    GlobIdn(Vec<String>),
    Expr(Expr),
    Type(String),
    Bound(Bound),
    Block(Box<ST>),
    Stmt(Stmt),
}

struct Parser<'a> {
    lexemes: Vec<Lexeme>,
    state: &'a mut State,
}

impl<'a> Parser<'_> {
    fn done(&self) -> bool {
        self.lexemes.is_empty()
    }

    fn consume_idn(&mut self) -> Spanned<Option<String>> {
        let lexeme = self.lexemes.remove(0);
        match lexeme.spanned {
            LexemeType::Idn(idn) => Spanned::new(Some(idn), lexeme.span),
            _ => Spanned::new(None, lexeme.span),
        }
    }

    fn parse_type(&mut self) -> Option<STN> {
        match self.consume_idn() {
            Spanned {
                spanned: Some(idn),
                span,
            } => Some(STN::new(ST::Idn(idn), span)),
            Spanned {
                spanned: None,
                span,
            } => {
                self.state
                    .errors
                    .push(SdwErr::from_pos(ParseErrors::ExpectedType, span));
                None
            }
        }
    }
}

pub fn parse(state: &mut State, lexemes: Vec<Lexeme>) {
    let mut parser = Parser { state, lexemes };
    while !parser.done() {
        // parsing logic
        let r#type = parser.parse_type();
        if r#type.is_none() {
            continue;
        }
        println!("ty: {:?}", r#type.unwrap().spanned);
    }

    // HACK: how else can we do this?
}
