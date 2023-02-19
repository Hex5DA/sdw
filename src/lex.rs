use anyhow::{bail, Context, Result};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Modifier {
    Mutable,
    Dynamic,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Keyword {
    Fn,
    Return,
    Variable,
    Modifier(Modifier),
    Coercion,
    If,
    Else,
}

impl Keyword {
    fn from_str(from: &String) -> Result<Self> {
        Ok(match from.as_str() {
            "fn" => Keyword::Fn,
            "return" => Keyword::Return,
            "var" => Keyword::Variable,
            "as" => Keyword::Coercion,
            "mut" => Keyword::Modifier(Modifier::Mutable),
            "dyn" => Keyword::Modifier(Modifier::Dynamic),
            "if" => Keyword::If,
            "else" => Keyword::Else,
            _ => bail!("Unknown keyword parsed, '{from}'"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Literal {
    Integer(i64), // TODO: Add support for negative numbers
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Lexeme {
    Keyword(Keyword),
    Idn(String),
    Literal(Literal),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Newline,
    Delimiter,
    Equals,
    Bang,
    Addition,
    Subtraction,
    Multiplication,
    Division,
    AngleLeft,
    AngleRight,
}

impl Lexeme {
    fn from_char(from: char) -> Result<Self> {
        Ok(match from {
            '{' => Lexeme::OpenBrace,
            '}' => Lexeme::CloseBrace,
            '(' => Lexeme::OpenParen,
            ')' => Lexeme::CloseParen,
            ';' => Lexeme::Newline,
            ',' => Lexeme::Delimiter,
            '=' => Lexeme::Equals,
            '!' => Lexeme::Bang,
            '+' => Lexeme::Addition,
            '-' => Lexeme::Subtraction,
            '*' => Lexeme::Multiplication,
            '/' => Lexeme::Division,
            '<' => Lexeme::AngleLeft,
            '>' => Lexeme::AngleRight,
            _ => bail!("Unknown symbol '{from}' encountered."),
        })
    }
}

struct LexBuffer {
    idx: i64,
    inp: String,
}

impl LexBuffer {
    fn at(&self, idx: i64) -> Result<char> {
        self.inp.chars().nth(idx as usize).context(format!(
            "Index out of bounds ({}/{})",
            idx,
            self.inp.len()
        ))
    }

    fn get(&self) -> Result<char> {
        self.at(self.idx)
    }

    fn peek(&self) -> Result<char> {
        self.at(self.idx + 1)
    }

    fn empty(&self) -> bool {
        self.inp.is_empty()
    }

    fn next(&mut self) {
        self.idx += 1;
    }

    fn trim(&mut self, to: i64) -> Result<()> {
        self.inp = self
            .inp
            .get((to as usize)..)
            .context(format!("Index out of bounds ({}/{})", to, self.inp.len()))?
            .to_string();
        self.idx = 0;
        Ok(())
    }
}

pub fn lex(inp: String) -> Result<Vec<Lexeme>> {
    let mut buf = LexBuffer { inp, idx: 0 };
    let mut lexemes: Vec<Lexeme> = vec![];

    while !buf.empty() {
        while buf.get()?.is_ascii_alphabetic() {
            if !buf.peek()?.is_ascii_alphabetic() {
                let kw_idn = buf.inp[..(buf.idx as usize) + 1].to_string();
                buf.trim(buf.idx + 1)?;
                let lexeme = if let Ok(kw) = Keyword::from_str(&kw_idn) {
                    Lexeme::Keyword(kw)
                } else {
                    Lexeme::Idn(kw_idn)
                };
                lexemes.push(lexeme);
                break;
            }
            buf.next();
        }

        while buf.get()?.is_ascii_digit() {
            if !buf.peek()?.is_ascii_digit() {
                let numlit = buf.inp[..(buf.idx as usize) + 1].to_string();
                buf.trim(buf.idx + 1)?;
                let lexeme = Lexeme::Literal(Literal::Integer(numlit.parse().unwrap()));
                lexemes.push(lexeme);
                break;
            }
            buf.next();
        }

        if let Ok(lexeme) = Lexeme::from_char(buf.get()?) {
            lexemes.push(lexeme);
        }

        buf.next();
        buf.trim(buf.idx)?;
    }

    Ok(lexemes)
}
