use crate::errors::LexErrors;
use crate::prelude::*;

/// Sub-enum of lexemes; possible keyword
#[derive(Debug)]
pub enum Keywords {
    Fn,
    Return,
}

/// Structure for holding different literals
#[derive(Debug)]
pub enum Literal {
    Integer(i64),
}

/// The master list of possible lexemes.
#[derive(Debug)]
pub enum LexemesTypes {
    Keyword(Keywords),
    Literal(Literal),
    Idn(String),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
}

#[derive(Debug)]
pub struct Lexeme {
    ty: LexemesTypes,
    pos: PosInfo,
}

pub fn lex(_raw: &String) -> crate::errors::Result<Vec<Lexeme>> {
    Err(ShadowError::new(
        LexErrors::UnrecognisedToken("[".to_string()),
        1,
        12,
        1,
    ))
}
