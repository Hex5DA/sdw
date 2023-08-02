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
    ($result:expr) => {{
        match $result {
            Success(success) => success,
            Fail => {
                return Ok(Fail);
            }
        }
    }};
}

pub mod prelude {
    pub use super::*;
}

// TODO: directives (needs rethinking conceptually?)
// TODO: `struct` / `union` initialisation syntax

// TODO: some of the codebase will need retrofitting to work with these
#[allow(dead_code)]
type Path = Vec<Segment>;
#[allow(dead_code)]
pub enum Segment {
    Idn(String),
    Wildcard,
}

type Idn = String;
type Type = String;

#[derive(Debug)]
pub enum PrimType {
    Int,
    Unt,
    Float,
    Bool,
    String,
}

// TODO: unspan these
#[derive(Debug)]
pub enum Bound {
    Prim(Spanned<PrimType>),
    Struct(Option<Vec<(Spanned<Bound>, Spanned<Idn>)>>),
    Union(Option<Vec<(Spanned<Bound>, Spanned<Idn>)>>),
    Alias(Spanned<String>),
    Pointer(Box<Spanned<Self>>),
    FnPtr {
        args: Vec<Spanned<Type>>,
        return_type: Spanned<Type>,
    },
}

type ExprSelf = Box<Spanned<Expr>>;
#[derive(Debug)]
pub enum Expr {
    IntLiteral(i64),
    BoolLiteral(bool),
    Variable(String),
    UnaryNot(ExprSelf),
    UnaryNeg(ExprSelf),
    UnaryPos(ExprSelf),
    SubExpr(ExprSelf),
    FnCall(Spanned<String>, Vec<ExprSelf>),
    BiOp(ExprSelf, BiOps, ExprSelf),
    /// &[var]
    Referal(ExprSelf),
    /// *[var]
    Indir(ExprSelf),
    ObjMember(Spanned<String>, Spanned<String>),
    Cond {
        condition: ExprSelf,
        then: Spanned<Block>,
        elifs: Vec<(ExprSelf, Spanned<Block>)>,
        r#else: Option<Spanned<Block>>,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Block {
    /// are subexpressions
    /// may contain statements
    /// value is equal to the last expression,
    ///   or is `Discard` if the last element is a statement,
    ///   or if the block is empty `{}`
    pub stmts: Vec<Spanned<Stmt>>,
    pub tail: Option<Box<Spanned<Expr>>>,
}

impl From<Vec<Spanned<Stmt>>> for Block {
    fn from(stmts: Vec<Spanned<Stmt>>) -> Self {
        Self { stmts, tail: None }
    }
}

#[derive(Debug)]
pub enum BiOps {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitOr,
    BitAnd,
    BitNot,
    BitXor,
    BitRshift,
    BitLShift,
    LogOr,
    LogAnd,
    LogNot,
    Eq,
    NEq,
    Gr,
    Ls,
    GrEq,
    LsEq,
}

#[derive(Debug)]
pub enum Stmt {
    Fn {
        return_type: Spanned<Type>,
        name: Spanned<Idn>,
        parameters: Vec<(Spanned<Type>, Spanned<Idn>)>,
        body: Box<Spanned<Block>>,
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
        expr: Option<Spanned<Expr>>,
    },
    VarDec {
        name: Spanned<Idn>,
        initialiser: Spanned<Expr>,
    },
    VarRes {
        name: Spanned<Idn>,
        updated: Spanned<Expr>,
    },
    Type {
        name: Spanned<Idn>,
        bound: Spanned<Bound>,
    },
    Discard {
        expr: Expr,
    },
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
            Err(SdwErr::from_pos(ParseErrors::TkStackEmpty, self.last_span))
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

    // TODO: how parse expressions thing maybe yes?? :)
    fn parse_block(&mut self) -> Result<Block> {
        let mut stmts = Vec::new();
        loop {
            // escaping this way feels camp, but i think it's reasonable
            // braces aren't overloaded beyond block scope delimters, so
            // this is a reasonable assumption??
            if self.done() || self.peek()?.spanned == LexemeType::RBrace {
                break;
            }

            match self.parse_stmt()? {
                Success(leaf) => stmts.push(leaf),
                Fail => continue,
            }
        }

        Ok(stmts.into())
    }

    fn parse_block_braces(&mut self) -> Return<Block> {
        let start = self.next_span()?;
        attempt!(
            self,
            self.expect(LexemeType::LBrace)?,
            ParseErrors::BlockNotOpened
        );
        let block = self.parse_block()?;
        attempt!(
            self,
            self.expect(LexemeType::RBrace)?,
            ParseErrors::BlockNotClosed
        );
        let span = Span::from_to(start, self.last_span);
        Ok(Success(Spanned::new(block, span)))
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
                                ParseErrors::TkStackEmptyDiag(Box::new(ParseErrors::NoFnBodyStub)),
                                self.last_span,
                            )
                        })?;

                        stub = Some(lexeme_after.spanned == LexemeType::Semi);
                        break;
                    }
                }

                let stub = stub.ok_or(SdwErr::from_pos(
                    ParseErrors::TkStackEmptyDiag(Box::new(ParseErrors::NoFnBodyStub)),
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
                    body = Some(Box::new(attempt!(self.parse_block_braces()?)));
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
                let block = Box::new(self.parse_block()?);
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
                    Some(attempt!(
                        self,
                        self.parse_expr()?,
                        ParseErrors::NoReturnExpr
                    ))
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
                let initialiser = attempt!(self, self.parse_expr()?, ParseErrors::NoLetInitialiser);
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

                let updated = attempt!(self, self.parse_expr()?, ParseErrors::NoLetInitialiser);
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
                let bound = attempt!(self.parse_bound()?);

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
                "int" => Spanned::new(Bound::Prim(Spanned::new(PrimType::Int, start)), start),
                "unt" => Spanned::new(Bound::Prim(Spanned::new(PrimType::Unt, start)), start),
                "float" => Spanned::new(Bound::Prim(Spanned::new(PrimType::Float, start)), start),
                "bool" => Spanned::new(Bound::Prim(Spanned::new(PrimType::Bool, start)), start),
                "string" => Spanned::new(Bound::Prim(Spanned::new(PrimType::String, start)), start),
                _ => Spanned::new(Bound::Alias(Spanned::new(potential, start)), start),
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
                        let bound = attempt!(self.parse_bound()?);
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
                let bound = attempt!(self.parse_bound()?);
                let span = Span::from_to(start, self.last_span);
                Spanned::new(Bound::Pointer(Box::new(bound)), span)
            }
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

                attempt!(
                    self,
                    self.expect(LexemeType::RParen)?,
                    ParseErrors::FnArgListNotClosed
                );
                attempt!(
                    self,
                    self.expect(LexemeType::Dash)?,
                    ParseErrors::FnPtrTyArrow
                );
                attempt!(
                    self,
                    self.expect(LexemeType::RAng)?,
                    ParseErrors::FnPtrTyArrow
                );

                let end = self.next_span()?;
                let span = Span::from_to(start, end);
                let return_type =
                    attempt!(self, self.parse_type()?, ParseErrors::ExpectedFnPtrReturnTy);

                Spanned::new(Bound::FnPtr { args, return_type }, span)
            }
            _ => attempt!(self, Fail, ParseErrors::InvalidBound),
        }))
    }

    fn parse_expr(&mut self) -> Return<Expr> {
        self.parse_expr_rbp(0)
    }

    fn parse_expr_rbp(&mut self, rbp: usize) -> Return<Expr> {
        let mut left = attempt!(self.nud()?);
        while self.peek()?.spanned.prec() > rbp {
            left = attempt!(self.led(left)?);
        }

        Ok(Success(left))
    }

    fn nud(&mut self) -> Return<Expr> {
        let start = self.next_span()?;
        Ok(Success(match self.next()?.spanned {
            LexemeType::Intlit(il) => Spanned::new(Expr::IntLiteral(il), start),
            LexemeType::BoolLit(bl) => Spanned::new(Expr::BoolLiteral(bl), start),
            LexemeType::Cross => {
                let expr = attempt!(self.parse_expr()?);
                let span = Span::from_to(start, expr.span);
                Spanned::new(Expr::UnaryPos(Box::new(expr)), span)
            }
            LexemeType::Dash => {
                let expr = attempt!(self.parse_expr()?);
                let span = Span::from_to(start, expr.span);
                Spanned::new(Expr::UnaryNeg(Box::new(expr)), span)
            }
            LexemeType::Bang => {
                let expr = attempt!(self.parse_expr()?);
                let span = Span::from_to(start, expr.span);
                Spanned::new(Expr::UnaryNot(Box::new(expr)), span)
            }
            LexemeType::Amp => {
                let expr = attempt!(self.parse_expr()?);
                let span = Span::from_to(start, expr.span);
                Spanned::new(Expr::Referal(Box::new(expr)), span)
            }
            LexemeType::Ast => {
                let expr = attempt!(self.parse_expr()?);
                let span = Span::from_to(start, expr.span);
                Spanned::new(Expr::Indir(Box::new(expr)), span)
            }
            LexemeType::LParen => {
                let expr = attempt!(self.parse_expr()?);
                let span = Span::from_to(start, self.next_span()?);
                attempt!(
                    self,
                    self.expect(LexemeType::RParen)?,
                    ParseErrors::SubExprNotClosed
                );
                Spanned::new(Expr::SubExpr(Box::new(expr)), span)
            }
            LexemeType::Idn(name) => match self.peek()?.spanned {
                LexemeType::LParen => {
                    self.next().unwrap(); // consume the parenthesis
                    let name = Spanned::new(name, start);
                    let mut args = Vec::new();
                    if self.peek()?.spanned != LexemeType::RParen {
                        while self.peek()?.spanned != LexemeType::RParen {
                            let arg = attempt!(self.parse_expr()?);
                            args.push(Box::new(arg));

                            if let Fail = self.expect(LexemeType::Comma)? {
                                if self.peek()?.spanned == LexemeType::RParen {
                                    break;
                                }
                                attempt!(self, Fail, ParseErrors::StubNoArgDel);
                            }
                        }
                    }

                    let end = self.next_span()?;
                    let span = Span::from_to(start, end);
                    attempt!(
                        self,
                        self.expect(LexemeType::RParen)?,
                        ParseErrors::FnArgListNotClosed
                    );
                    Spanned::new(Expr::FnCall(name, args), span)
                }
                LexemeType::Period => {
                    self.next().unwrap(); // consume the period
                    let name = Spanned::new(name, start);
                    let end = self.next_span()?;
                    let member = attempt!(self, self.consume_idn()?, ParseErrors::NoMemberName);
                    let span = Span::from_to(start, end);
                    Spanned::new(Expr::ObjMember(name, member), span)
                }
                _ => Spanned::new(Expr::Variable(name), start),
            },
            LexemeType::If => {
                // TODO: try removing the `()` dependency for conditions.
                //       issue: how do we disambuiguiate blocks & conditionals??
                attempt!(
                    self,
                    self.expect(LexemeType::LParen)?,
                    ParseErrors::CondAmbigious
                );
                let condition = Box::new(attempt!(self.parse_expr()?));
                attempt!(
                    self,
                    self.expect(LexemeType::RParen)?,
                    ParseErrors::CondAmbigious
                );
                // TODO: this is an expression that literally won't work cause
                //       blocks can't parse expressions yet lol
                let then = attempt!(self.parse_block_braces()?);

                let mut elifs = Vec::new();
                let mut r#else = None;
                while self.peek()?.spanned == LexemeType::Else {
                    if self.peek()?.spanned == LexemeType::If {
                        attempt!(
                            self,
                            self.expect(LexemeType::LParen)?,
                            ParseErrors::CondAmbigious
                        );
                        let condition = Box::new(attempt!(self.parse_expr()?));
                        attempt!(
                            self,
                            self.expect(LexemeType::RParen)?,
                            ParseErrors::CondAmbigious
                        );

                        let then = attempt!(self.parse_block_braces()?);
                        elifs.push((condition, then));
                        continue;
                    }

                    r#else = Some(attempt!(self.parse_block_braces()?));
                    break;
                }

                let span = Span::from_to(start, self.last_span);
                Spanned::new(
                    Expr::Cond {
                        condition,
                        then,
                        elifs,
                        r#else,
                    },
                    span,
                )
            }
            LexemeType::LBrace => todo!("blocks"),
            _ => unimplemented!("error here"),
        }))
    }

    fn led(&mut self, left: Spanned<Expr>) -> Return<Expr> {
        let next = self.next()?;
        let biop = match (&next.spanned, self.peek()?.spanned) {
            // the most specific patterns (2 lexemes) must come first.
            (LexemeType::RAng, LexemeType::RAng) => BiOps::BitRshift,
            (LexemeType::LAng, LexemeType::LAng) => BiOps::BitLShift,
            (LexemeType::Bar, LexemeType::Bar) => BiOps::LogOr,
            (LexemeType::Amp, LexemeType::Amp) => BiOps::LogAnd,
            (LexemeType::Equals, LexemeType::Equals) => BiOps::Eq,
            (LexemeType::Bang, LexemeType::Equals) => BiOps::NEq,
            (LexemeType::RAng, LexemeType::Equals) => BiOps::GrEq,
            (LexemeType::LAng, LexemeType::Equals) => BiOps::LsEq,
            (LexemeType::RAng, _) => BiOps::Gr,
            (LexemeType::LAng, _) => BiOps::Ls,
            (LexemeType::Cross, _) => BiOps::Add,
            (LexemeType::Dash, _) => BiOps::Sub,
            (LexemeType::Ast, _) => BiOps::Mul,
            (LexemeType::FSlash, _) => BiOps::Div,
            (LexemeType::Perc, _) => BiOps::Mod,
            (LexemeType::Bar, _) => BiOps::BitOr,
            (LexemeType::Amp, _) => BiOps::BitAnd,
            (LexemeType::Tilde, _) => BiOps::BitNot,
            (LexemeType::Caret, _) => BiOps::BitXor,
            (uk, _) => attempt!(self, Fail, ParseErrors::UnknownOperator(uk.clone())),
        };

        let right = attempt!(self.parse_expr_rbp(next.spanned.prec())?);
        let span = Span::from_to(left.span, right.span);
        Ok(Success(Spanned::new(
            Expr::BiOp(Box::new(left), biop, Box::new(right)),
            span,
        )))
    }
}

impl LexemeType {
    fn prec(&self) -> usize {
        match self {
            LexemeType::Cross | LexemeType::Dash => 5,
            LexemeType::Ast | LexemeType::FSlash => 10,
            _ => 0,
        }
    }
}

pub fn parse(state: &mut State, lexemes: Vec<Lexeme>) -> Result<Block> {
    let mut parser = Parser::new(lexemes, state);
    parser.parse_block()
}
