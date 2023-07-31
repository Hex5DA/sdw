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
pub enum PrimType {
    Int,
    Unt,
    Float,
    Bool,
    String,
}

#[derive(Debug)]
pub enum Bound {
    Prim(PrimType),
    Struct(Option<Vec<(Spanned<Bound>, Spanned<Idn>)>>),
    Union(Option<Vec<(Spanned<Bound>, Spanned<Idn>)>>),
    Alias(String),
    Pointer(Box<Self>),
    FnPtr {
        args: Vec<Spanned<Type>>, 
        return_type: Spanned<Type>,
    },
}

#[derive(Debug)]
pub enum Expr {
    IntLiteral(i64),
}

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
    Loop {
        block: Box<Block>,
    },
    Label {
        name: Spanned<Idn>,
    },
    Goto {
        name: Spanned<Idn>,
    },
    Return {
        // TODO: `STN` or `Expr`?
        expr: Option<Box<STN>>,
    },
    VarDec {
        name: Spanned<Idn>,
        // TODO: ditto
        initialiser: Box<STN>,
    },
    VarRes {
        name: Spanned<Idn>,
        // TODO: ditto
        updated: Box<STN>,
    },
    Type {
        name: Spanned<Idn>,
        bound: Box<STN>,
    },
}

pub type STN = Spanned<ST>;
pub type Block = Vec<STN>;
#[derive(Debug)]
pub enum ST {
    Expr(Expr),
    Bound(Bound),
    Stmt(Stmt),
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
        loop {
            // escaping this way feels camp, but i think it's reasonable
            // * as it stands * `{}` aren't overloaded beyond block scope delimters, so
            // this is a reasonable assumption??
            if self.done() || self.peek()?.spanned == LexemeType::RBrace {
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
        self.consume_idn()
    }

    fn parse_stmt(&mut self) -> Return<Stmt> {
        let next = self.next()?;
        let start = next.span;

        Ok(match next.spanned {
            LexemeType::Fn => {
                // fn int addTwo(int arg1, int arg2) { [body] };
                // ^^ ^^^ ^^^^^^^
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

                if stub {
                    // if the parameter list is empty, we skip this
                    if self.peek()?.spanned != LexemeType::RParen {
                        loop {
                            let r#type =
                                attempt!(self, self.parse_type()?, ParseErrors::ExpectedArgType);
                            stub_parameters.push(r#type);

                            if let Fail = self.expect(LexemeType::Comma)? {
                                // if there is neither a comma nor an RParen..
                                if let LexemeType::RParen = self.peek()?.spanned {
                                    break;
                                }
                                // .. error ..
                                attempt!(self, Fail, ParseErrors::StubNoArgDel);
                                // .. because we would be in a situation like this:
                                // fn int addTwo(int string);
                                //                  ^ (no comma!)
                            }
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
                        ParseErrors::BlockNotOpened
                    );
                    body = Some(Box::new(self.parse()?));
                    attempt!(
                        self,
                        self.expect(LexemeType::RBrace)?,
                        ParseErrors::BlockNotClosed
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
            LexemeType::Mc => unimplemented!("macros aren't implemented, and won't be for a while"),
            LexemeType::Loop => {
                attempt!(
                    self,
                    self.expect(LexemeType::LBrace)?,
                    ParseErrors::BlockNotOpened
                );
                let block = Box::new(self.parse()?);
                attempt!(
                    self,
                    self.expect(LexemeType::RBrace)?,
                    ParseErrors::BlockNotClosed
                );
                let end = self.next_span()?;
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );
                let span = Span::from_to(start, end);
                Success(Spanned::new(Stmt::Loop { block }, span))
            }
            LexemeType::At => {
                let name = attempt!(self, self.consume_idn()?, ParseErrors::LabelName);
                let end = self.next_span()?;
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );
                let span = Span::from_to(start, end);
                Success(Spanned::new(Stmt::Label { name }, span))
            }
            LexemeType::Goto => {
                attempt!(
                    self,
                    self.expect(LexemeType::At)?,
                    ParseErrors::LabelNamePrefixedAt
                );
                let name = attempt!(self, self.consume_idn()?, ParseErrors::GotoNeedLabel);
                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );

                Success(Spanned::new(Stmt::Goto { name }, span))
            }
            LexemeType::Return => {
                let expr = if let LexemeType::Semi = self.peek()?.spanned {
                    None
                } else {
                    let expr = attempt!(self, self.parse_expr()?, ParseErrors::NoReturnExpr);
                    let stn = Box::new(STN::new(ST::Expr(expr.spanned), expr.span));
                    Some(stn)
                };

                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );

                Success(Spanned::new(Stmt::Return { expr }, span))
            }
            LexemeType::Let => {
                let name = attempt!(self, self.consume_idn()?, ParseErrors::NoVarName);
                attempt!(
                    self,
                    self.expect(LexemeType::Equals)?,
                    ParseErrors::ExpectedEquals
                );
                // TODO: stub declaration?
                //       (`let foo;`)
                let expr = attempt!(self, self.parse_expr()?, ParseErrors::NoLetInitialiser);
                let initialiser = Box::new(STN::new(ST::Expr(expr.spanned), expr.span));
                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );

                Success(Spanned::new(Stmt::VarDec { name, initialiser }, span))
            }
            LexemeType::Idn(idn) => {
                let name = Spanned::new(idn, next.span);
                if let Fail = self.expect(LexemeType::Equals)? {
                    // HACK: what do we want here?  don't want to just error out,
                    //       cause i think a lone `Idn(..)` is more indicative of a greater error
                    return Ok(Fail);
                }

                let expr = attempt!(self, self.parse_expr()?, ParseErrors::NoLetInitialiser);
                let updated = Box::new(STN::new(ST::Expr(expr.spanned), expr.span));
                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );

                Success(Spanned::new(Stmt::VarRes { name, updated }, span))
            }
            LexemeType::Type => {
                let name = attempt!(self, self.consume_idn()?, ParseErrors::NoTypeDecName);
                let bound = attempt!(self, self.parse_bound()?, ParseErrors::NoBound);
                let bound = Box::new(STN::new(ST::Bound(bound.spanned), bound.span));

                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                attempt!(
                    self,
                    self.expect(LexemeType::Semi)?,
                    ParseErrors::StmtsEndWithSemi
                );

                Success(Spanned::new(Stmt::Type { name, bound }, span))
            }
            // TODO: verify correct behaviour.
            _ => Fail,
        })
    }

    fn parse_bound(&mut self) -> Return<Bound> {
        let next = self.next()?;
        let start = next.span;

        Ok(Success(match next.spanned {
            LexemeType::Idn(potential) => match potential.as_str() {
                "int" => Spanned::new(Bound::Prim(PrimType::Int), start),
                "unt" => Spanned::new(Bound::Prim(PrimType::Unt), start),
                "float" => Spanned::new(Bound::Prim(PrimType::Float), start),
                "bool" => Spanned::new(Bound::Prim(PrimType::Bool), start),
                "string" => Spanned::new(Bound::Prim(PrimType::String), start),
                _ => Spanned::new(Bound::Alias(potential), start),
            },
            LexemeType::Struct | LexemeType::Union => {
                let members = if self.peek()?.spanned != LexemeType::LBrace {
                    None
                } else {
                    attempt!(
                        self,
                        self.expect(LexemeType::LBrace)?,
                        ParseErrors::BlockNotOpened
                    );
                    let mut members = Vec::new();
                    while let Fail = self.expect(LexemeType::RBrace)? {
                        // TODO: error?
                        let bound = attempt!(self, self.parse_bound()?, ParseErrors::NoBound);
                        let name = attempt!(self, self.consume_idn()?, ParseErrors::NoMemberName);
                        members.push((bound, name));
                        let _ = self.expect(LexemeType::Comma);
                    }

                    Some(members)
                };

                let span = Span::from_to(start, self.last_span);
                Spanned::new(
                    match next.spanned {
                        LexemeType::Struct => Bound::Struct(members),
                        LexemeType::Union => Bound::Union(members),
                        _ => unreachable!(),
                    },
                    span,
                )
            }
            LexemeType::Amp => {
                let bound = attempt!(self, self.parse_bound()?, ParseErrors::NoBound);
                let span = Span::from_to(start, self.last_span);
                Spanned::new(Bound::Pointer(Box::new(bound.spanned)), span)
            },
            LexemeType::LParen => {
                let mut args = Vec::new();
                if self.peek()?.spanned != LexemeType::RParen {
                    while self.peek()?.spanned != LexemeType::RParen {
                        let r#type = attempt!(self, self.parse_type()?, ParseErrors::FnPtrTyNoType);
                        args.push(r#type);

                        if let Fail = self.expect(LexemeType::Comma)? {
                            if self.peek()?.spanned == LexemeType::RParen {
                                break;
                            }
                            attempt!(self, Fail, ParseErrors::StubNoArgDel);
                        }
                    }
                }

                attempt!(self, self.expect(LexemeType::RParen)?, ParseErrors::FnArgListNotClosed);
                attempt!(self, self.expect(LexemeType::Dash)?, ParseErrors::FnPtrTyArrow);
                attempt!(self, self.expect(LexemeType::RAng)?, ParseErrors::FnPtrTyArrow);

                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                let return_type = attempt!(self, self.parse_type()?, ParseErrors::ExpectedFnPtrReturnTy);

                Spanned::new(Bound::FnPtr { args, return_type }, span)
            },

            _ => attempt!(self, Fail, ParseErrors::InvalidBound),
        }))
    }

    fn parse_expr(&mut self) -> Return<Expr> {
        let _ = self.next();
        Ok(Success(Spanned::new(Expr::IntLiteral(0), self.last_span)))
    }
}

pub fn parse(state: &mut State, lexemes: Vec<Lexeme>) -> Result<Vec<STN>> {
    let mut parser = Parser::new(lexemes, state);
    parser.parse()
}
