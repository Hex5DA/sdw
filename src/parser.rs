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

enum Attempt<T> {
    Success(T),
    Fail,
}

use Attempt::*;
type Return = Result<Attempt<STN>>;

struct Parser<'a> {
    lexemes: Vec<Lexeme>,
    state: &'a mut State,
    last_span: Span,
}

impl<'a> Parser<'_> {
    fn done(&self) -> bool {
        self.lexemes.is_empty()
    }

    fn consume_idn(&mut self) -> Result<Spanned<Attempt<String>>> {
        if self.lexemes.is_empty() {
            return Err(SdwErr::from_pos(ParseErrors::TkStackEmpty(Box<ParseErrors::ExpectedIdn>), self.last_span));
        }

        let lexeme = self.lexemes.remove(0);
        Ok(match lexeme.spanned {
            LexemeType::Idn(idn) => Spanned::new(Success(idn), lexeme.span),
            _ => Spanned::new(Fail, lexeme.span),
        })
    }

    fn parse_type(&mut self) -> Return {
        Ok(match self.consume_idn()? {
            Spanned {
                spanned: Success(idn),
                span,
            } => Success(STN::new(ST::Idn(idn), span)),
            Spanned {
                spanned: No,
                span,
            } => {
                self.state
                    .errors
                    .push(SdwErr::from_pos(ParseErrors::ExpectedType, span));
                Fail
            }
        })
    }

    fn parse_stmt(&mut self) -> Option<STN> {

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
