use crate::parse::prelude::*;
use crate::prelude::*;

// Pratt parsing:

pub enum Expression {
    Literal(i64),
    Binary(Box<Expression>, BinOp, Box<Expression>),
}

pub enum BinOp {
    Addition,
    Multiplication
}



