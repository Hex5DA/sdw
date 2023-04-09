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
    IntLit(i64),
    BoolLit(bool),
    Variable(String),
    BinOp(ExprWrapper, BinOpTypes, ExprWrapper),
    Comp(ExprWrapper, CompTypes, ExprWrapper),
    Group(ExprWrapper),
}

#[derive(Debug, Clone)]
pub enum BinOpTypes {
    Add,
    Sub,
    Div,
    Mul,
}

#[derive(Debug, Clone)]
pub enum CompTypes {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

impl LexemeTypes {
    fn prec(&self) -> u64 {
        match self {
            LexemeTypes::Cross | LexemeTypes::Dash => 10,
            LexemeTypes::Asterisk | LexemeTypes::FSlash => 20,
            // TODO: is this a good precedence?? idk lol
            LexemeTypes::AngleLeft | LexemeTypes::AngleRight | LexemeTypes::Bang | LexemeTypes::Equals => 30,
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
                span = Span::from_to(left.span, right.span);
                Expression::BinOp(left.into(), next.try_into().unwrap(), right)
            }
            LexemeTypes::Equals | LexemeTypes::Bang | LexemeTypes::AngleLeft | LexemeTypes::AngleRight => {
                let cmp = self.parse_comp(next.clone())?;
                let right: ExprWrapper = self._parse_expr(next.ty.prec())?.into();
                span = Span::from_to(left.span, right.span);
                Expression::Comp(left.into(), cmp, right)
            },
            ty => return Err(ShadowError::from_pos(ParseErrors::UnknownOperator(ty), next.span)),
        };
        Ok(Spanned::new(span, expr))
    }

    /// this is called under the assumption that the next tokens are valid comparisons
    fn parse_comp(&mut self, next: Lexeme) -> Result<CompTypes> {
        Ok(match next.ty {
            LexemeTypes::AngleRight => {
                if self.peek()?.ty == LexemeTypes::Equals {
                    self.consume(LexemeTypes::Equals).unwrap();
                    CompTypes::GreaterThanEqualTo
                } else {
                    CompTypes::GreaterThan
                }
            }
            LexemeTypes::AngleLeft => {
                if self.peek()?.ty == LexemeTypes::Equals {
                    self.consume(LexemeTypes::Equals).unwrap();
                    CompTypes::LessThanEqualTo
                } else {
                    CompTypes::LessThan
                }
            }
            LexemeTypes::Equals => { self.consume(LexemeTypes::Equals)?; CompTypes::Equal },
            LexemeTypes::Bang => { self.consume(LexemeTypes::Equals)?; CompTypes::NotEqual },
            _ => unreachable!(),
        })
    }
}
