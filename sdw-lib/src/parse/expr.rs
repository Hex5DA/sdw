use super::ParseBuffer;
use crate::errors::ParseErrors;
use crate::prelude::*;

// a 'lil impl i wrote up to get a grasp on pratt parsing.
// <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b5dbccaf078a030a6cc3a7b6f3e0bb3b>

type ExprWrapper = Spanned<Box<Expression>>;

impl From<Spanned<Expression>> for ExprWrapper {
    fn from(from: Spanned<Expression>) -> ExprWrapper {
        ExprWrapper::new(from.span, Box::new(from.inner))
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    // literals
    IntLit(i64),
    BoolLit(bool),
    Variable(String),
    BinOp(ExprWrapper, BinOpTypes, ExprWrapper),
    // others
    Group(ExprWrapper),
}

#[derive(Debug, Clone)]
pub enum BinOpTypes {
    Add,
    Sub,
    Div,
    Mul,
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

impl TryFrom<Lexeme> for BinOpTypes {
    type Error = ShadowError;
    fn try_from(from: Lexeme) -> Result<BinOpTypes> {
        Ok(match from.ty {
            LexemeTypes::Cross => BinOpTypes::Add,
            LexemeTypes::Dash => BinOpTypes::Sub,
            LexemeTypes::Asterisk => BinOpTypes::Mul,
            LexemeTypes::FSlash => BinOpTypes::Div,
            ty => return Err(ShadowError::from_pos(ParseErrors::InvalidExpressionLHS(ty), from.span)),
        })
    }
}

impl ParseBuffer {
    pub fn parse_expr(&mut self) -> Result<Spanned<Expression>> {
        self._parse_expr(0)
    }

    fn _parse_expr(&mut self, rbp: u64) -> Result<Spanned<Expression>> {
        let next = self.pop()?;
        let mut left = self.nud(next)?;
        while self.peek()?.ty.prec() > rbp {
            left = self.led(left)?;
        }

        Ok(left)
    }

    fn nud(&mut self, next: Lexeme) -> Result<Spanned<Expression>> {
        Ok(Spanned::new(
            next.span,
            match next.ty {
                LexemeTypes::Literal(l) => match l {
                    Literals::Boolean(b) => Expression::BoolLit(b),
                    Literals::Integer(n) => Expression::IntLit(n),
                },
                LexemeTypes::Idn(name) => Expression::Variable(name),
                LexemeTypes::OpenParen => {
                    let expr = self.parse_expr()?;
                    let end = self.consume(LexemeTypes::CloseParen)?.span;
                    Expression::Group(Spanned::new(Span::from_to(next.span, end), Box::new(expr.inner)))
                }
                ty => return Err(ShadowError::from_pos(ParseErrors::InvalidExpressionLHS(ty), next.span)),
            },
        ))
    }

    fn led(&mut self, left: Spanned<Expression>) -> Result<Spanned<Expression>> {
        let next = self.pop()?;
        let span;
        let expr = match next.ty {
            LexemeTypes::Cross | LexemeTypes::Dash | LexemeTypes::FSlash | LexemeTypes::Asterisk => {
                let right: ExprWrapper = self._parse_expr(next.ty.prec())?.into();
                println!("right: {:#?}", right);
                span = Span::from_to(left.span, right.span);
                Expression::BinOp(left.into(), next.try_into().unwrap(), right)
            }
            ty => return Err(ShadowError::from_pos(ParseErrors::UnknownOperator(ty), next.span)),
        };
        Ok(Spanned::new(span, expr))
    }
}
