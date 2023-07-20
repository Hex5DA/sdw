use crate::prelude::*;

macro_rules! attempt {
    ($parser:expr, $result:expr, $err:expr) => {{
        match $result {
            Success(success) => success,
            Fail => {
                $parser
                    .state
                    .errors
                    .push(SdwErr::from_pos($err, $parser.last_span));
                return Ok(Fail);
            }
        }
    }};
}

type Idn = String;
type GlobIdn = Vec<String>;
type Type = String;

#[derive(Debug)]
pub enum Bound {}
#[derive(Debug)]
pub enum Expr {}

#[derive(Debug)]
pub enum Stmt {
    Fn {
        return_type: Spanned<Type>,
        name: Spanned<Idn>,
        parameters: Vec<(Spanned<Type>, Spanned<Idn>)>,
        body: Box<STN>,
    },
    Stub {
        return_type: Spanned<Type>,
        name: Spanned<Idn>,
        parameters: Vec<Spanned<Type>>,
    },
}

pub type STN = Spanned<ST>;
#[derive(Debug)]
pub enum ST {
    Expr(Expr),
    Bound(Bound),
    Stmt(Stmt),

    Block(Box<ST>),
}

enum Attempt<T> {
    Success(T),
    Fail,
}

use Attempt::*;
type Return<T> = Result<Attempt<Spanned<T>>>;

struct Parser<'a> {
    lexemes: Vec<Lexeme>,
    state: &'a mut State,
    last_span: Span,
}

impl<'a> Parser<'a> {
    fn new(lexemes: Vec<Lexeme>, state: &'a mut State) -> Parser<'a> {
        Self {
            lexemes,
            state,
            last_span: Span::default(),
        }
    }

    fn done(&self) -> bool {
        self.lexemes.is_empty()
    }

    fn next(&mut self) -> Result<Lexeme> {
        if self.lexemes.is_empty() {
            return Err(SdwErr::from_pos(
                ParseErrors::TkStackEmpty(Box::new(ParseErrors::ExpectedIdn)),
                self.last_span,
            ));
        }

        let next = self.lexemes.remove(0);
        self.last_span = next.span;
        Ok(next)
    }

    fn next_span(&self) -> Result<Span> {
        if self.lexemes.is_empty() {
            return Err(SdwErr::from_pos(
                ParseErrors::TkStackEmpty(Box::new(ParseErrors::NoMoreSpans)),
                self.last_span,
            ));
        }

        Ok(self.lexemes[0].span)
    }

    fn consume_idn(&mut self) -> Return<String> {
        let lexeme = self.next()?;
        Ok(match lexeme.spanned {
            LexemeType::Idn(idn) => Success(Spanned::new(idn, lexeme.span)),
            _ => Fail,
        })
    }

    fn parse_type(&mut self) -> Return<Type> {
        Ok(match self.consume_idn()? {
            Fail => {
                self.state
                    .errors
                    .push(SdwErr::from_pos(ParseErrors::ExpectedType, self.last_span));
                Fail
            }
            success => success,
        })
    }

    fn expect(&mut self, r#type: LexemeType) -> Return<LexemeType> {
        let lexeme = self.next()?;
        Ok(match lexeme.spanned {
            success @ r#type => Success(Spanned::new(success, lexeme.span)),
            _ => Fail,
        })
    }

    fn parse_stmt(&mut self) -> Return<Stmt> {
        let next = self.next()?;

        match next.spanned {
            LexemeType::Fn => {
                let start = self.next_span()?;
                let return_type =
                    attempt!(self, self.parse_type()?, ParseErrors::MissingFnReturnType);
                let name = attempt!(self, self.consume_idn()?, ParseErrors::MissingFnIdn);
                attempt!(
                    self,
                    self.expect(LexemeType::LParen)?,
                    ParseErrors::NoFnArgs
                );

                // fn int adder(int arg1, int arg2) [ body ];
                // fn int adder();
                // ^^^^^^^^^^^^^

                // TODO: reach end of token stack whilst parsing?
                let mut stub;
                for (idx, lexeme) in self.lexemes.iter().enumerate() {
                    // HACK: could this ever produce a false positive?
                    //       (or, add error checking incase it ever should - but how?)
                    if let LexemeType::RParen = lexeme.spanned {
                        let lexeme_after = self.lexemes.get(idx + 1).ok_or_else(|| {
                            SdwErr::from_pos(
                                ParseErrors::TkStackEmpty(Box::new(ParseErrors::NoFnBodyStub)),
                                self.last_span,
                            )
                        })?;

                        stub = lexeme_after.spanned == LexemeType::Semi;
                    }
                }

                let parameters = Vec::new();
                if stub {
                    // parse stub argument list
                    for lexeme in self.lexemes {
                        if let LexemeType::RParen = lexeme.spanned {
                            break;
                        }

                        let r#type =
                            attempt!(self, self.parse_type()?, ParseErrors::ExpectedArgType);
                        parameters.push(r#type);
                        let _ = self.expect(LexemeType::Comma);
                    }

                    // note: this could still error, if we fell off whilst iterating
                    attempt!(self, self.expect(LexemeType::RParen)?, ParseErrors::FnArgListNotClosed);
                    let end = self.next_span()?;
                    attempt!(self, self.expect(LexemeType::Semi)?, ParseErrors::StmtsEndWithSemi);

                    STN::new(
                        ST::Stmt(Stmt::Stub {
                            return_type,
                            name,
                            parameters,
                        }),
                        Span::from_to(start, end),
                    )
                } else {
                    // parse function argument list
                    let end = self.next_span()?; // TODO: remove
                    let span = Span::from_to(start, end);
                    STN::new(
                        ST::Stmt(Stmt::Fn {
                            return_type,
                            name,
                            parameters: (),
                            body: (),
                        }),
                        span,
                    )
                }
            }
            _ => todo!(),
        };

        todo!()
    }
}

pub fn parse(state: &mut State, lexemes: Vec<Lexeme>) -> Result<Vec<STN>> {
    let mut parser = Parser::new(lexemes, state);
    let mut stmts = Vec::new();

    while !parser.done() {
        // TODO: recovery??
        match parser.parse_stmt()? {
            Success(leaf) => stmts.push(leaf),
            Fail => continue,
        }
    }
    Ok(stmts)
}
