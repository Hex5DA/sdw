use super::ParseBuffer;
use crate::errors::ParseErrors;
use crate::prelude::*;

// a 'lil impl i wrote up to get a grasp on pratt parsing.
// <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b5dbccaf078a030a6cc3a7b6f3e0bb3b>

#[derive(Debug, Clone)]
pub enum Expression {
    // literals
    IntLit(i64),
    BoolLit(bool),
    // binary operations
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    // others
    Variable(String),
    Group(Box<Expression>),
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

impl ParseBuffer {
    pub fn parse_expr(&mut self) -> Result<Expression> {
        self._parse_expr(0)
    }

    fn _parse_expr(&mut self, rbp: u64) -> Result<Expression> {
        let next = self.pop()?;
        let mut left = self.nud(next)?;
        while self.peek()?.ty.prec() > rbp {
            left = self.led(left)?;
        }

        Ok(left)
    }

    fn nud(&mut self, next: Lexeme) -> Result<Expression> {
        Ok(match next.ty {
            LexemeTypes::Literal(l) => match l {
                Literals::Boolean(b) => Expression::BoolLit(b),
                Literals::Integer(n) => Expression::IntLit(n),
            },
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
        Ok(match next.ty {
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
