use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    static ref IDN_REGEX: Regex = Regex::new(r"[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexemeType {
    // arithmetic operators
    /// +
    Cross,
    /// -
    Dash,
    /// *
    Ast,
    /// /
    FSlash,
    /// %
    Perc,

    // logical & bitwise operators
    /// |
    Bar,
    /// &
    Amp,
    /// ~
    Tilde,
    /// ^
    Caret,
    /// >
    RAng,
    /// <
    LAng,
    /// !
    Bang,
    /// =
    Equals,

    // braces
    /// {
    LBrace,
    /// }
    RBrace,
    /// (
    LParen,
    /// )
    RParen,
    /// [
    RBrack,
    /// ]
    LBrack,

    // punctuation
    /// #
    Hash,
    /// @
    At,
    /// :
    Colon,
    /// ;
    Semi,
    /// .
    Period,
    /// ,
    Comma,

    /// string of characters (`[_ | a-z | a-Z][_ | a-z | A-Z | 0-9]?*`)
    Idn(String),
    Intlit(u64), // TODO: integer sizes??
    BoolLit(bool),

    // keywords
    // procedures
    /// `fn`
    Fn,
    /// `mc`
    Mc,
    /// `return`
    Return,
    /// `state`
    State,

    // control flow
    /// `if`
    If,
    /// `else`
    Else,
    /// `goto`
    Goto,
    /// `loop`
    Loop,

    // misc
    /// `struct`
    Struct,
    /// `type`
    Type,
    /// `let`
    Let,
    /// `mod`
    Mod,
}

pub struct UnknownLexeme(String);
impl FromStr for LexemeType {
    type Err = UnknownLexeme;
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match value {
            "+" => Self::Cross,
            "-" => Self::Dash,
            "*" => Self::Ast,
            "/" => Self::FSlash,
            "%" => Self::Perc,
            "|" => Self::Bar,
            "&" => Self::Amp,
            "~" => Self::Tilde,
            "^" => Self::Caret,
            ">" => Self::RAng,
            "<" => Self::LAng,
            "!" => Self::Bang,
            "=" => Self::Equals,
            "{" => Self::LBrace,
            "}" => Self::RBrace,
            "(" => Self::LParen,
            ")" => Self::RParen,
            "[" => Self::LBrack,
            "]" => Self::RBrack,
            "#" => Self::Hash,
            "@" => Self::At,
            ":" => Self::Colon,
            ";" => Self::Semi,
            "." => Self::Period,
            "," => Self::Comma,
            "fn" => Self::Fn,
            "mc" => Self::Mc,
            "return" => Self::Return,
            "state" => Self::State,
            "if" => Self::If,
            "else" => Self::Else,
            "goto" => Self::Goto,
            "loop" => Self::Loop,
            "struct" => Self::Struct,
            "type" => Self::Type,
            "let" => Self::Let,
            "mod" => Self::Mod,
            "true" => Self::BoolLit(true),
            "false" => Self::BoolLit(false),
            tok => {
                if IDN_REGEX.is_match(tok) {
                    Self::Idn(tok.to_string())
                } else if let Ok(num) = tok.parse::<u64>() {
                    Self::Intlit(num)
                } else {
                    return Err(UnknownLexeme(tok.to_string()));
                }
            }
        })
    }
}

pub type Lexeme = Spanned<LexemeType>;

struct LexBuffer {
    stream: String,
    position: Span,
    // idx is 1D
    idx: usize,
}

impl LexBuffer {
    fn new(stream: String) -> Self {
        Self {
            stream,
            position: Span::default(),
            idx: 0,
        }
    }

    fn done(&self) -> bool {
        self.stream.is_empty()
    }

    fn over(&self) -> char {
        self.stream.chars().nth(self.idx).unwrap_or_else(|| {
            panic!(
                "lexer: position OOB ({}/{})\nremaining content:{}",
                self.idx,
                self.stream.len(),
                self.stream
            );
        })
    }

    fn adv(&mut self, by: usize) {
        self.idx += by;
        self.position.ecol += by as u64;
    }

    fn eat(&mut self) -> String {
        let chunk = self.stream.drain(..self.idx).collect();
        self.position.sline = self.position.eline;
        self.position.scol = self.position.ecol;
        self.idx = 0;
        chunk
    }

    fn tok(&mut self) -> Result<Lexeme> {
        let chunk = self.eat();
        let span = Span {
            ecol: self.position.ecol + 1,
            eline: self.position.eline + 1,
            ..self.position
        };
        let r#type = chunk.parse().map_err(|err: UnknownLexeme| {
            SdwErr::from_pos(LexErrors::UnrecognisedToken(err.0), span)
        })?;

        Ok(Lexeme {
            spanned: r#type,
            span,
        })
    }
}

macro_rules! err {
    ($state:expr, $result:expr, $name:ident => $stmt:expr) => {{
        match $result {
            Ok($name) => $stmt,
            Err(err) => $state.errors.push(err),
        }
    }};
}

pub fn lex(state: &mut State, raw: &str) -> Vec<Lexeme> {
    let mut lexemes = Vec::new();
    let mut buffer = LexBuffer::new(raw.to_owned());

    while !buffer.done() {
        if buffer.over().is_ascii_alphabetic() || buffer.over() == '_' {
            buffer.adv(1);
            while buffer.over().is_ascii_alphanumeric() || buffer.over() == '_' {
                buffer.adv(1);
            }

            err![state, buffer.tok(), ok => lexemes.push(ok)];
            continue;
        }

        if buffer.over().is_ascii_digit() {
            buffer.adv(1);
            while buffer.over().is_ascii_digit() {
                buffer.adv(1);
            }

            err![state, buffer.tok(), ok => lexemes.push(ok)];
            // lexemes.push(buffer.tok()?);
            continue;
        }

        if buffer.over().is_ascii_whitespace() {
            // HACK: escaping via `buffer.done()` feels camp, though i *think* it's reasonable?
            while !buffer.done() && buffer.over().is_ascii_whitespace() {
                buffer.adv(1);
                if buffer.eat() == "\n" {
                    buffer.position.eline += 1;
                    buffer.position.ecol = 0;
                }
            }

            continue;
        }

        buffer.adv(1);
        err![state, buffer.tok(), ok => lexemes.push(ok)];
        // lexemes.push(buffer.tok()?);
    }

    lexemes
}
