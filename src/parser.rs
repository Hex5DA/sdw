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

pub mod prelude {
    pub use super::*;
}

type Idn = String;
#[allow(dead_code)]
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
        body: Box<Block>,
    },
    Stub {
        return_type: Spanned<Type>,
        name: Spanned<Idn>,
        parameters: Vec<Spanned<Type>>,
    },
}

pub type STN = Spanned<ST>;
pub type Block = Vec<STN>;
#[derive(Debug)]
pub enum ST {
    Expr(Expr),
    Bound(Bound),
    Stmt(Stmt),

    Block(Box<ST>),
}

#[derive(Debug)]
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

    // not for external usage!
    fn _tk_empty(&self) -> Result<()> {
        if self.lexemes.is_empty() {
            Err(SdwErr::from_pos(
                ParseErrors::TkStackEmpty(Box::new(ParseErrors::ExpectedIdn)),
                self.last_span,
            ))
        } else {
            Ok(())
        }
    }

    /// always consumes
    fn next(&mut self) -> Result<Lexeme> {
        self._tk_empty()?;

        let next = self.lexemes.remove(0);
        self.last_span = next.span;
        Ok(next)
    }

    /// never consumes
    fn peek(&self) -> Result<Lexeme> {
        self._tk_empty()?;
        Ok(self.lexemes[0].clone())
    }

    /// only consumes if `r#type` matches
    fn expect(&mut self, r#type: LexemeType) -> Return<LexemeType> {
        let lexeme = self.peek()?;
        Ok(if lexeme.spanned == r#type {
            self.next().unwrap();
            Success(Spanned::new(lexeme.spanned, lexeme.span))
        } else {
            Fail
        })
    }

    fn next_span(&self) -> Result<Span> {
        self._tk_empty()?;
        // we don't call `self.peek()` to avoid a clone
        Ok(self.lexemes[0].span)
    }

    fn consume_idn(&mut self) -> Return<String> {
        let lexeme = self.next()?;
        Ok(match lexeme.spanned {
            LexemeType::Idn(idn) => Success(Spanned::new(idn, lexeme.span)),
            _ => Fail,
        })
    }

    fn parse(&mut self) -> Result<Block> {
        let mut stmts = Vec::new();
        while !self.done() {
            // escaping this way feels camp, but i think it's reasonable
            // * as it stands * `{}` aren't overloaded beyond block scope delimters, so
            // this is a reasonable assumption??
            if let Spanned {
                spanned: LexemeType::RBrace,
                ..
            } = self.peek()?
            {
                break;
            }

            match self.parse_stmt()? {
                Success(leaf) => stmts.push(STN::new(ST::Stmt(leaf.spanned), leaf.span)),
                Fail => continue,
            }
        }

        Ok(stmts)
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

    fn parse_stmt(&mut self) -> Return<Stmt> {
        let next = self.next()?;

        Ok(match next.spanned {
            LexemeType::Fn => {
                // fn int addTwo(int arg1, int arg2) { [body] };
                // ^^ ^^^ ^^^^^^^
                let start = self.next_span()?;
                let return_type =
                    attempt!(self, self.parse_type()?, ParseErrors::MissingFnReturnType);
                let name = attempt!(self, self.consume_idn()?, ParseErrors::MissingFnIdn);
                attempt!(
                    self,
                    self.expect(LexemeType::LParen)?,
                    ParseErrors::NoFnArgs
                );

                // test if we are parsing a stub or bodied function
                // (we find the first `)`, and check if the following lexeme is a `;`)
                let mut stub = None;
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

                        stub = Some(lexeme_after.spanned == LexemeType::Semi);
                        break;
                    }
                }

                let stub = stub.ok_or(SdwErr::from_pos(
                    ParseErrors::TkStackEmpty(Box::new(ParseErrors::NoFnBodyStub)),
                    self.last_span,
                ))?;

                let mut stub_parameters = Vec::new();
                let mut body_parameters = Vec::new();

                // fn int addTwo(int arg1, int arg2) { [body] };
                //               ^^^ ^^^^  ^^^ ^^^^

                // TODO: properly comment this shit lol
                //       (good luck figuring this out, too!)
                //       (no i'm not doing it now whilst i still understand it)
                //       (suffer, future will)
                if stub {
                    loop {
                        let r#type =
                            attempt!(self, self.parse_type()?, ParseErrors::ExpectedArgType);
                        stub_parameters.push(r#type);

                        if let Fail = self.expect(LexemeType::Comma)? {
                            if let LexemeType::RParen = self.peek()?.spanned {
                                break;
                            }
                            attempt!(self, Fail, ParseErrors::StubNoArgDel);
                        }
                    }
                } else {
                    loop {
                        if let LexemeType::RParen = self.peek()?.spanned {
                            break;
                        }
                        let r#type =
                            attempt!(self, self.parse_type()?, ParseErrors::ExpectedArgType);
                        let pm_name =
                            attempt!(self, self.consume_idn()?, ParseErrors::ExpectedArgIdn);
                        body_parameters.push((r#type, pm_name));
                        // we break by checking for `RParen`, not `Comma`,
                        // so we don't care about the result
                        let _ = self.expect(LexemeType::Comma);
                    }
                }

                // fn int addTwo(int arg1, int arg2) { [body] };
                //                                 ^
                // note: this could still error, if we fell off whilst iterating
                attempt!(
                    self,
                    self.expect(LexemeType::RParen)?,
                    ParseErrors::FnArgListNotClosed
                );

                // fn int addTwo(int arg1, int arg2) { [body] };
                //                                   ^ [^^^^] ^
                let mut body = None;
                if !stub {
                    attempt!(
                        self,
                        self.expect(LexemeType::LBrace)?,
                        ParseErrors::FnRequiresBody
                    );
                    body = Some(Box::new(self.parse()?));
                    attempt!(
                        self,
                        self.expect(LexemeType::RBrace)?,
                        ParseErrors::FnBodyNotClosed
                    );
                }

                // fn int addTwo(int arg1, int arg2) { [body] };
                //                                             ^
                let end = self.next_span()?;
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );
                let span = Span::from_to(start, end);

                Success(if stub {
                    Spanned::new(
                        Stmt::Stub {
                            return_type,
                            name,
                            parameters: stub_parameters,
                        },
                        span,
                    )
                } else {
                    Spanned::new(
                        Stmt::Fn {
                            return_type,
                            name,
                            parameters: body_parameters,
                            // (safe - this branch only executes when body is `Some`)
                            body: body.unwrap(),
                        },
                        span,
                    )
                })
            }
            // TODO: delete
            unknown => todo!("unknown: {:?}", unknown),
        })
    }
}

pub fn parse(state: &mut State, lexemes: Vec<Lexeme>) -> Result<Vec<STN>> {
    let mut parser = Parser::new(lexemes, state);
    parser.parse()
}
