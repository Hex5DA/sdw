use thiserror::Error;
use crate::common::PosInfo;

pub type Result<T> = std::result::Result<T, ShadowError>;

#[derive(Debug)]
pub struct ShadowError {
    ty: ErrType,
    pub pos: PosInfo,
}

impl std::fmt::Display for ShadowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[SDW E/{}]",
            match self.ty {
                ErrType::Lex(_) => "L",
            },
        )?;
        writeln!(f, "{}", self.ty)?;
        writeln!(
            f,
            "error occurred at line {}, character {}.",
            self.pos.line, self.pos.column
        )?;
        Ok(())
    }
}

fn repeat_char(ch: char, len: usize) -> String {
    std::iter::repeat(ch).take(len).collect::<String>()
}

impl ShadowError {
    pub fn verbose(&self, raw: &String) {
        println!("[ .. ]");
        println!(
            "{}",
            raw.split("\n")
                .collect::<Vec<&str>>()
                .get((self.pos.line - 1) as usize)
                .expect("an error was reported on a line that does not exist")
        );
        println!(
            "{}{} - error occured here!",
            repeat_char(' ', (self.pos.column - 1) as usize),
            repeat_char('^', self.pos.length as usize)
        );
        println!("[ .. ]");
    }

    pub fn new<T: Into<ErrType>>(err: T, line: u64, column: u64, length: u64) -> Self {
        Self {
            ty: err.into(),
            pos: PosInfo {
                line,
                column,
                length,
            },
        }
    }
}

/*

--

[E/L] malformed token

unrecognised token '[' - perhaps you meant '('?
error occured at line 3, character 4.

[ .. ]
fn int main[) {
           ^^ - error occurred here!
[ .. ]

--

information needed:
- type of error (parse, lex, IR, semantic)
- error number
- error diagnostic
- error line number / character position
- access to raw file content

*/

#[derive(Debug)]
pub enum ErrType {
    Lex(LexErrors),
}

impl std::fmt::Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lex(lexerr) => lexerr,
            }
        )
    }
}

#[derive(Error, Debug)]
pub enum LexErrors {
    #[error("an unrecognised token was occured: {0}")]
    UnrecognisedToken(String),
}

impl Into<ErrType> for LexErrors {
    fn into(self) -> ErrType {
        ErrType::Lex(self)
    }
}

