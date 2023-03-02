use crate::common::Span;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ShadowError>;

#[derive(Debug)]
pub struct ShadowError {
    ty: ErrType,
    pub span: Span,
}

impl std::fmt::Display for ShadowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[SDW E/{}]",
            match self.ty {
                ErrType::Lex(_) => "L",
                ErrType::Parse(_) => "P",
            },
        )?;
        writeln!(f, "{}", self.ty)?;
        writeln!(
            f,
            "error occurred at line {}, character {}.",
            self.span.line + 1,
            self.span.column + 1
        )?;
        Ok(())
    }
}

fn repeat_char(ch: char, len: usize) -> String {
    std::iter::repeat(ch).take(len).collect::<String>()
}

impl ShadowError {
    pub fn verbose(&self, raw: &str) {
        println!("[ .. ]");
        println!(
            "{}",
            raw.split('\n')
                .collect::<Vec<&str>>()
                .get(self.span.line as usize)
                .expect("an error was reported on a line that does not exist")
        );
        println!(
            "{}{} - error occured here!",
            repeat_char(' ', self.span.column as usize),
            repeat_char('^', self.span.length as usize)
        );
        println!("[ .. ]");
    }

    pub fn new<T: Into<ErrType>>(err: T, line: u64, column: u64, length: u64) -> Self {
        Self {
            ty: err.into(),
            span: Span {
                line,
                column,
                length,
            },
        }
    }

    pub fn from_pos<T: Into<ErrType>>(err: T, span: Span) -> Self {
        Self {
            ty: err.into(),
            span,
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
    Parse(ParseErrors),
}

impl std::fmt::Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lex(lexerr) => format!("{}", lexerr),
                Self::Parse(parseerr) => format!("{}", parseerr),
            }
        )
    }
}

#[derive(Error, Debug)]
pub enum LexErrors {
    #[error("an unrecognised token was occured: '{0}'")]
    UnrecognisedToken(String),
}

impl From<LexErrors> for ErrType {
    fn from(other: LexErrors) -> ErrType {
        ErrType::Lex(other)
    }
}

#[derive(Error, Debug)]
pub enum ParseErrors {
    #[error("an example error to demonstrate how parsing errors will be handled. content: {0}")]
    Example(String),
}

impl From<ParseErrors> for ErrType {
    fn from(other: ParseErrors) -> ErrType {
        ErrType::Parse(other)
    }
}
