use crate::prelude::*;

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
    Semicolon,
    /// .
    Period,
    /// ,
    Comma,

    /// string of characters (`[_ | a-z | a-Z][_ | a-z | A-Z | 0-9]?*`)
    Idn(String),

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

impl TryFrom<String> for LexemeType {
    type Error = SdwError;
    fn try_from(value: String) -> Result<Self> {
        Ok(match value.as_str() {
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
            _ => todo!()
        })
    }
}

pub type Lexeme = Spanned<LexemeType>;

struct LexBuffer {
    stream: String,
    position: Span,
    idx: usize,
}

impl LexBuffer {
    fn new(stream: String) -> Self {
        Self { stream, position: Span::blank(), idx: 0 }
    }

    fn done(&self) -> bool {
        self.stream.is_empty()
    }

    fn over(&self) -> char {
        self.stream.chars().nth(self.idx).unwrap_or_else(|| {
            panic!("lexer: position OOB ({}/{})\nremaining content:{}", self.idx, self.stream.len(), self.stream);
        })
    }

    fn adv(&mut self, by: usize) {
        self.idx += by;
        self.position.ecol += by as u64;
    }

    fn eat(&mut self) -> String {
        let chunk = self.stream.drain(..self.idx).collect();
        self.idx = 0;
        chunk
    }

    fn tok(&mut self) -> Lexeme {
        let chunk = self.eat();
        let r#type = chunk.into();

        Lexeme {
            r#type, span
        }
    }
}


pub fn lex(raw: &String) -> Vec<Lexeme> {
    let mut lexemes = Vec::new();
    let mut buffer = LexBuffer::new(raw.clone());

    while !buffer.done() {
        if buffer.over().is_alphabetic() || buffer.over() == '_' {
            buffer.adv(1);
            while buffer.over().is_alphanumeric() || buffer.over() == '_' {
                buffer.adv(1);
            }
            
            lexemes.push(buffer.eat().into());
            continue;
        }
    }

    lexemes
}

