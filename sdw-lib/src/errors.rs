use crate::common::Span;
use owo_colors::OwoColorize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ShadowError>;

#[derive(Debug)]
pub struct ShadowError {
    pub ty: ErrType,
    pub span: Span,
}

fn repeat_char(ch: char, len: usize) -> String {
    std::iter::repeat(ch).take(len).collect::<String>()
}

impl ShadowError {
    fn header(&self) {
        eprint!(
            "{} ",
            format!(
                "[SDW E/{}]",
                match self.ty {
                    ErrType::Lex(_) => "L",
                    ErrType::Parse(_) => "P",
                },
            )
            .red()
        );
        eprintln!("{}", self.ty);
        eprintln!(
            "{} error occurred at {}, {}.",
            "->".blue(),
            ("line ".to_owned() + &(self.span.line + 1).to_string()).blue(),
            ("character ".to_owned() + &(self.span.column + 1).to_string()).blue()
        );
    }

    fn body(&self, raw: &str) {
        let lines = raw.split('\n').collect::<Vec<&str>>();
        if self.span.line > 1 {
            eprintln!("[ .. ]")
        };
        eprintln!(
            "{}",
            lines
                .get(self.span.line as usize)
                .expect("an error was reported on a line that does not exist.. somehow")
        );
        eprintln!(
            "{}{} {}",
            repeat_char(' ', self.span.column as usize),
            repeat_char('^', self.span.length as usize).red(),
            "- error occured here!".red()
        );
        if self.span.line as usize == lines.len() {
            eprintln!("[ .. ]")
        };
    }

    pub fn print(&self, raw: &str) {
        self.header();
        self.body(raw);
    }

    pub fn new<T: Into<ErrType>>(err: T, line: u64, column: u64, length: u64) -> Self {
        Self {
            ty: err.into(),
            span: Span { line, column, length },
        }
    }

    pub fn from_pos<T: Into<ErrType>>(err: T, span: Span) -> Self {
        Self { ty: err.into(), span }
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
    #[error("an unrecognised type was encountered: '{0}'")]
    UnrecognisedType(String),
}

impl From<LexErrors> for ErrType {
    fn from(other: LexErrors) -> ErrType {
        ErrType::Lex(other)
    }
}

#[derive(Error, Debug)]
pub enum ParseErrors {
    #[error("the token stack was empty")]
    TokenStackEmpty,
    #[error("an unexpected token was encountered: {0} (expected {1})")]
    UnexpectedTokenEncountered(String, String),
    #[error("the token '{0}' was unrecognised whilst parsing a statement. this error sucks lmao")]
    UnknownStartOfStatement(crate::lex::LexemeTypes),
}

impl From<ParseErrors> for ErrType {
    fn from(other: ParseErrors) -> ErrType {
        ErrType::Parse(other)
    }
}
