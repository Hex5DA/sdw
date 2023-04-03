use crate::errors::LexErrors;
use crate::prelude::*;

use lazy_static::lazy_static;
use regex::Regex;

use std::fmt::{self, Display};

lazy_static! {
    static ref IDN_RE: Regex = Regex::new(r"[a-zA-Z][a-zA-Z0-9_]*").unwrap();
}

/// sub-enum of lexemes; possible keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keywords {
    Fn,
    Return,
    Let,
}

impl Keywords {
    fn new(from: &str) -> Option<Self> {
        Some(match from {
            "fn" => Keywords::Fn,
            "return" => Keywords::Return,
            "let" => Keywords::Let,
            _ => return None,
        })
    }
}

impl Display for Keywords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Fn => "fn",
                Self::Return => "return",
                Self::Let => "let",
            }
        )
    }
}

/// structure for holding different literals
/// eg. inetger literals: `10`, string literals, `"bobirty"`, ect..
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literals {
    Integer(i64),
}

impl Display for Literals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Integer(i) => i,
            }
        )
    }
}
/// the master list of possible lexemes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexemeTypes {
    /// see keywords enum; possible keywords
    Keyword(Keywords),
    /// see literal enum; possible literal values
    Literal(Literals),
    /// an identifier. must match the regex `[a-zA-Z][a-zA-Z0-9_]*`
    Idn(String),
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenBrace,
    //// }
    CloseBrace,
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// =
    Equals,
    /// +
    Cross,
    /// -
    Dash,
    /// /
    FSlash,
    /// *
    Asterisk,
}

impl LexemeTypes {
    fn new(from: &str) -> Option<Self> {
        use LexemeTypes::*;
        Some(match from {
            "(" => OpenParen,
            ")" => CloseParen,
            "{" => OpenBrace,
            "}" => CloseBrace,
            ";" => Semicolon,
            "," => Comma,
            "=" => Equals,
            "+" => Cross,
            "-" => Dash,
            "/" => FSlash,
            "*" => Asterisk,
            other => {
                if let Some(kw) = Keywords::new(other) {
                    Keyword(kw)
                } else if let Ok(num) = other.parse::<i64>() {
                    Literal(self::Literals::Integer(num))
                } else if IDN_RE.is_match(other) {
                    Idn(other.to_string())
                } else {
                    return None;
                }
            }
        })
    }
}

impl Display for LexemeTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let binding; // appeasing the borrow checker.
        write!(
            f,
            "'{}'",
            match self {
                Self::Keyword(kw) => {
                    binding = kw.to_string();
                    binding.as_str()
                }
                Self::Literal(lt) => {
                    binding = lt.to_string();
                    binding.as_str()
                }
                Self::Idn(idn) => idn.as_str(),
                Self::OpenParen => "(",
                Self::CloseParen => ")",
                Self::OpenBrace => "{",
                Self::CloseBrace => "}",
                Self::Semicolon => ";",
                Self::Comma => ",",
                Self::Equals => "=",
                Self::Cross => "+",
                Self::Dash => "-",
                Self::FSlash => "/",
                Self::Asterisk => "*",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Lexeme {
    pub ty: LexemeTypes,
    pub span: Span,
}

impl Lexeme {
    fn new(lb: &LexBuffer, raw_token: &String) -> Result<Lexeme> {
        let length = raw_token.len() as u64;
        let span = Span {
            line: lb.span.line,
            column: lb.span.column - length,
            end_line: lb.span.line,
            end_col: lb.span.column - length,
        };
        let ty = LexemeTypes::new(raw_token)
            .ok_or_else(|| ShadowError::from_pos(LexErrors::UnrecognisedToken(raw_token.clone()), span))?;
        Ok(Lexeme { ty, span })
    }
}

/// simple buffer to make handling the input easier
struct LexBuffer {
    working: String,
    position: usize,
    span: Span,
}

impl LexBuffer {
    fn adv(&mut self, by: u64) {
        self.position += by as usize;
        self.span.column += by;
    }

    fn over(&self) -> char {
        self.working.chars().nth(self.position).unwrap_or_else(|| {
            panic!(
                "position OOB; ({}/{})\n{:?}",
                self.position,
                self.working.len(),
                self.working
            )
        })
    }

    fn eat(&mut self) -> String {
        let ret = self.working.drain(..self.position).collect();
        self.position = 0;
        ret
    }

    fn done(&self) -> bool {
        self.working.is_empty()
    }
}

pub fn lex(raw: &str) -> Result<Vec<Lexeme>> {
    let mut lb = LexBuffer {
        working: raw.to_owned(),
        position: 0,
        span: Span::default(),
    };
    let mut lexemes: Vec<Lexeme> = Vec::new();

    while !lb.done() {
        // strings of continous characters
        if lb.over().is_ascii_alphabetic() {
            while lb.over().is_ascii_alphabetic() {
                lb.adv(1);
            }
            let kw_idn = lb.eat();
            lexemes.push(Lexeme::new(&lb, &kw_idn)?);
            continue;
        }

        // strings of numbers
        if lb.over().is_ascii_digit() {
            while lb.over().is_ascii_digit() {
                lb.adv(1);
            }
            let num_lit = lb.eat();
            lexemes.push(Lexeme::new(&lb, &num_lit)?);
            continue;
        }

        // skip whitespace
        if lb.over().is_ascii_whitespace() {
            while !lb.working.is_empty() && lb.over().is_ascii_whitespace() {
                lb.adv(1);
                // ... but take note of newlines
                if lb.eat() == "\n" {
                    lb.span.line += 1;
                    lb.span.column = 0;
                }
            }
            continue;
        }

        lb.adv(1);
        let raw_token = &lb.eat();
        lexemes.push(Lexeme::new(&lb, raw_token)?);
    }

    Ok(lexemes)
}

#[cfg(test)]
mod tests {
    use super::LexemeTypes::*;
    use super::*;

    #[test]
    fn fn_dec() {
        let input = "fn int main() {\n    return 4;\n}";
        let lexemes = lex(input);
        assert!(lexemes.is_ok(), "error in the lexer");
        assert_eq!(
            lexemes.unwrap().iter().map(|l| l.ty.clone()).collect::<Vec<LexemeTypes>>(),
            vec![
                Keyword(Keywords::Fn),
                Idn("int".to_string()),
                Idn("main".to_string()),
                OpenParen,
                CloseParen,
                OpenBrace,
                Keyword(Keywords::Return),
                Literal(Literals::Integer(4)),
                Semicolon,
                CloseBrace
            ]
        );
    }
}
