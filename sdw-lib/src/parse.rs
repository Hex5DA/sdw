use self::expr::*;
use crate::errors::{LexErrors, ParseErrors};
use crate::prelude::*;

pub mod prelude {
    pub use super::expr::Expression;
    pub use super::{Block, Node, Spanned, Type};
}

pub mod expr {
    use crate::errors::ParseErrors;
    use super::{Parser, Spanned};
    use crate::prelude::*;

    // a 'lil impl i wrote up to get a grasp on pratt parsing.
    // <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b5dbccaf078a030a6cc3a7b6f3e0bb3b>

    #[derive(Debug, Clone)]
    pub enum Expression {
        // literals
        IntLit(i64),
        // binary operations
        Add(Box<Expression>, Box<Expression>),
        Sub(Box<Expression>, Box<Expression>),
        Mul(Box<Expression>, Box<Expression>),
        Div(Box<Expression>, Box<Expression>),
        // others
        Variable(String),
        Group(Box<Expression>),
    }

    impl Expression {
        // currently this is just calculating / evaluating
        pub fn eval(&self) -> i64 {
            match self {
                Expression::Add(o1, o2) => o1.eval() + o2.eval(),
                Expression::Sub(o1, o2) => o1.eval() - o2.eval(),
                Expression::Mul(o1, o2) => o1.eval() * o2.eval(),
                Expression::Div(o1, o2) => o1.eval() / o2.eval(),
                Expression::Group(inner) => inner.eval(),
                Expression::IntLit(i) => *i,
                Expression::Variable(name) => panic!("TODO: need to simplify expressions here. {}", name),
            }
        }
    }

    impl LexemeTypes {
        fn prec(&self) -> u64 {
            match self {
                LexemeTypes::Cross | LexemeTypes::Dash => 10,
                LexemeTypes::Asterisk | LexemeTypes::FSlash => 20,
                _ => 0,
            }
        }
    }

    impl Parser {
        pub fn parse_expr(&mut self) -> Result<Expression> {
            self._parse_expr(0)
        }

        fn _parse_expr(&mut self, rbp: u64) -> Result<Expression> {
            let next = self.pop()?;
            let mut left = self.nud(next)?;
            while self.peek()?.inner.ty.prec() > rbp {
                left = self.led(left)?;
            }

            Ok(left)
        }

        fn nud(&mut self, next: Spanned<Lexeme>) -> Result<Expression> {
            Ok(match next.inner.ty {
                LexemeTypes::Literal(Literal::Integer(n)) => Expression::IntLit(n),
                LexemeTypes::Idn(name) => Expression::Variable(name),
                LexemeTypes::OpenParen => {
                    let expr = self.parse_expr()?;
                    self.consume(LexemeTypes::CloseParen)?;
                    Expression::Group(Box::new(expr))
                }
                ty => return Err(ShadowError::from_pos(ParseErrors::InvalidExpressionLHS(ty), next.span)),
            })
        }

        fn led(&mut self, left: Expression) -> Result<Expression> {
            let next = self.pop()?;
            Ok(match next.inner.ty {
                // WARNING: LOOKING FOR PROLONGED PERIODS WILL CAUSE EYE-BLEED.
                //          READER DISCRETION ADVISED
                ty @ LexemeTypes::Cross => Expression::Add(Box::new(left), Box::new(self._parse_expr(ty.prec())?)),
                ty @ LexemeTypes::Dash => Expression::Sub(Box::new(left), Box::new(self._parse_expr(ty.prec())?)),
                ty @ LexemeTypes::FSlash => Expression::Div(Box::new(left), Box::new(self._parse_expr(ty.prec())?)),
                ty @ LexemeTypes::Asterisk => Expression::Mul(Box::new(left), Box::new(self._parse_expr(ty.prec())?)),
                ty => return Err(ShadowError::from_pos(ParseErrors::UnknownOperator(ty), next.span)),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Void,
}

impl Type {
    fn from_string(other: String, span: Span) -> Result<Type> {
        Ok(match other.as_str() {
            "int" => Type::Int,
            "void" => Type::Void,
            _ => return Err(ShadowError::from_pos(LexErrors::UnrecognisedType(other), span)),
        })
    }
}

pub type Block = Vec<Box<Node>>;
#[derive(Debug)]
pub enum Node {
    Function {
        params: Vec<(String, Type)>,
        name: String,
        return_ty: Type,
        body: Block,
    },
    Return {
        expr: Option<Expression>,
    },
    VDec {
        name: String,
        init: Expression,
    },
}

#[derive(Debug)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
}

impl Spanned<Lexeme> {
    fn from_lexeme(lexeme: Lexeme) -> Spanned<Lexeme> {
        Spanned::<Lexeme> {
            span: lexeme.span,
            inner: lexeme,
        }
    }
}

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
#[derive(Debug)]
pub struct Parser {
    working: LexemeStream,
    lexemes: Vec<Lexeme>,
}

impl Parser {
    pub fn new(lexemes: Vec<Lexeme>) -> Self {
        assert!(!lexemes.is_empty(), "attempted to construct parser with no lexemes");
        Self {
            working: lexemes.clone().into(),
            lexemes,
        }
    }

    pub fn consume(&mut self, ty: LexemeTypes) -> Result<Spanned<Lexeme>> {
        let tk = self.pop()?;
        if tk.inner.ty == ty {
            Ok(tk)
        } else {
            Err(ShadowError::from_pos(
                ParseErrors::UnexpectedTokenEncountered(tk.inner.ty, ty),
                tk.span,
            ))
        }
    }

    fn peek(&mut self) -> Result<Spanned<Lexeme>> {
        match self.working.front() {
            Some(l) => Ok(Spanned::from_lexeme(l.clone())),
            None => Err(ShadowError::from_pos(
                ParseErrors::TokenStackEmpty,
                self.lexemes.last().unwrap().span,
            )),
        }
    }

    fn pop(&mut self) -> Result<Spanned<Lexeme>> {
        match self.working.pop_front() {
            Some(l) => Ok(Spanned::from_lexeme(l)),
            None => Err(ShadowError::from_pos(
                ParseErrors::TokenStackEmpty,
                self.lexemes.last().unwrap().span,
            )),
        }
    }

    fn eat_idn(&mut self) -> Result<(String, Spanned<Lexeme>)> {
        let tk = self.pop()?;
        match tk.inner.ty {
            LexemeTypes::Idn(ref s) => Ok((s.to_owned(), tk)),
            _ => Err(ShadowError::from_pos(
                ParseErrors::UnexpectedTokenEncountered(tk.inner.ty, LexemeTypes::Idn("<idn>".to_string())),
                tk.span,
            )),
        }
    }

    fn parse_fndef(&mut self) -> Result<Node> {
        let (ty, ty_l) = self.eat_idn()?;
        let (nm, _nm_l) = self.eat_idn()?;
        self.consume(LexemeTypes::OpenParen)?;
        let mut params = Vec::new();
        while let Some(Lexeme {
            ty: LexemeTypes::Idn(_),
            ..
        }) = self.working.front()
        {
            // why does rustfmt do this :sob:
            let (ty, ty_l) = self.eat_idn()?;
            let (nm, _nm_l) = self.eat_idn()?;
            params.push((nm, Type::from_string(ty, ty_l.span)?));
            if let Some(Lexeme {
                ty: LexemeTypes::Comma, ..
            }) = self.working.front()
            {
                // unwrap() is okay here.
                self.pop().unwrap();
            } else {
                break;
            }
        }
        self.consume(LexemeTypes::CloseParen)?;
        self.consume(LexemeTypes::OpenBrace)?;
        let body = _parse(self)?;
        self.consume(LexemeTypes::CloseBrace)?;
        Ok(Node::Function {
            params,
            name: nm,
            return_ty: Type::from_string(ty, ty_l.span)?,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Node> {
        let mut expr = None;
        // != is not supported afaik. so cope.
        if let LexemeTypes::Semicolon = self.peek()?.inner.ty {
        } else {
            expr = Some(self.parse_expr()?);
        }
        self.consume(LexemeTypes::Semicolon)?;
        Ok(Node::Return { expr })
    }

    fn parse_vdec(&mut self) -> Result<Node> {
        let (name, _) = self.eat_idn()?;
        self.consume(LexemeTypes::Equals)?;
        let init = self.parse_expr()?;
        self.consume(LexemeTypes::Semicolon)?;
        Ok(Node::VDec { name, init })
    }

    fn done(&self) -> bool {
        self.working.is_empty()
    }
}

fn _parse(parser: &mut Parser) -> Result<Block> {
    let mut block = Block::new();
    loop {
        // nb. done < peek; short-circuiting
        if parser.done() || parser.peek()?.inner.ty == LexemeTypes::CloseBrace {
            break;
        }

        let next = parser.pop()?;
        let node = match next.inner.ty {
            LexemeTypes::Keyword(kw) => match kw {
                Keywords::Fn => parser.parse_fndef()?,
                Keywords::Return => parser.parse_return()?,
                Keywords::Let => parser.parse_vdec()?,
            },
            LexemeTypes::CloseBrace => unreachable!(),
            _ => unreachable!(),
        };
        block.push(Box::new(node));
    }

    Ok(block)
}

pub fn parse(lexemes: Vec<Lexeme>) -> Result<Block> {
    let mut parser = Parser::new(lexemes);
    let root = _parse(&mut parser)?;
    Ok(root)
}
