#[derive(Debug)]
pub enum Keyword {
    Fn,
    Return,
}

impl Keyword {
    fn from_str(from: &String) -> Result<Keyword, &'static str> {
        Ok(match from.as_str() {
            "fn" => Keyword::Fn,
            "return" => Keyword::Return,
            _ => return Err("todo: proper errors"),
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Lexeme {
    Keyword(Keyword),
    Idn(String),
    NumLiteral(u64), // TODO: Add support for negative numbers
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Newline,
    Delimiter,
}

impl Lexeme {
    fn from_char(from: char) -> Result<Lexeme, &'static str> {
        Ok(match from {
            '{' => Lexeme::OpenBrace,
            '}' => Lexeme::CloseBrace,
            '(' => Lexeme::OpenParen,
            ')' => Lexeme::CloseParen,
            ';' => Lexeme::Newline,
            ',' => Lexeme::Delimiter,
            _ => return Err("TODO: better error handling"),
        })
    }
}

struct LexBuffer {
    idx: i64,
    inp: String,
}

impl LexBuffer {
    fn at(&self, idx: i64) -> char {
        self.inp.chars().nth(idx as usize).unwrap() // TODO: error handling
    }

    fn get(&self) -> char {
        self.at(self.idx)
    }

    fn peek(&self) -> char {
        self.at(self.idx + 1)
    }

    fn empty(&self) -> bool {
        self.inp.is_empty()
    }

    fn next(&mut self) {
        self.idx += 1;
    }

    fn trim(&mut self, to: i64) {
        self.inp = self.inp.get((to as usize)..).unwrap().to_string();
        self.idx = 0;
    }
}

pub fn lex(inp: String) -> Vec<Lexeme> {
    let mut buf = LexBuffer { inp, idx: 0 };
    let mut lexemes: Vec<Lexeme> = vec![];

    while !buf.empty() {
        while buf.get().is_ascii_alphabetic() {
            if !buf.peek().is_ascii_alphabetic() {
                let kw_idn = (&buf.inp[..(buf.idx as usize) + 1]).to_string();
                buf.trim(buf.idx + 1);
                let lexeme = if let Ok(kw) = Keyword::from_str(&kw_idn) { Lexeme::Keyword(kw) } else { Lexeme::Idn(kw_idn) };
                lexemes.push(lexeme);
                break;
            }
            buf.next();
        }

         while buf.get().is_ascii_digit() {
            if !buf.peek().is_ascii_digit() {
                let numlit = (&buf.inp[..(buf.idx as usize) + 1]).to_string();
                buf.trim(buf.idx + 1);
                let lexeme = Lexeme::NumLiteral(numlit.parse().unwrap());
                lexemes.push(lexeme);
                break;
            }
            buf.next();
        }

        if let Ok(lexeme) = Lexeme::from_char(buf.get()) {
            lexemes.push(lexeme);
        }

        buf.next();
        buf.trim(buf.idx);
    }

    lexemes
}
