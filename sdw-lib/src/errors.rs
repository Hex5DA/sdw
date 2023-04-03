use crate::common::Span;
use crate::lex::LexemeTypes;
use owo_colors::OwoColorize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ShadowError>;

#[derive(Debug)]
pub struct ShadowError {
    pub ty: ErrType,
    pub span: Option<Span>,
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
                    ErrType::Semantic(_) => "S",
                },
            )
            .red()
        );
        eprintln!("{}", self.ty);
        if let Some(span) = self.span {
            eprintln!(
                "{} error occurred at {}, {}.",
                "->".blue(),
                ("line ".to_owned() + &(span.line + 1).to_string()).blue(),
                ("character ".to_owned() + &(span.column + 1).to_string()).blue()
            );
        }
    }

    fn body(&self, raw: &str) {
        let span = self.span.unwrap(); // `body()` should only be called if a `Span` is present
        let lines = raw.split('\n').collect::<Vec<&str>>();
        if span.line > 1 {
            eprintln!("[ .. ]")
        };
        // idk if this works
        for line in span.line..=span.end_line {
            eprintln!(
                "{}",
                lines
                    .get(line as usize)
                    .expect("an error was reported on a line that does not exist.. somehow")
            );
            eprintln!(
                "{}{} {}",
                repeat_char(' ', span.column as usize),
                repeat_char('^', (span.end_col - span.column + 1) as usize).red(),
                "- error occured here!".red()
            );
        }
        if span.line as usize == lines.len() {
            eprintln!("[ .. ]")
        };
    }

    pub fn print(&self, raw: &str) {
        self.header();
        if self.span.is_some() {
            self.body(raw);
        }
    }

    pub fn from_pos<T: Into<ErrType>>(err: T, span: Span) -> Self {
        Self {
            ty: err.into(),
            span: Some(span),
        }
    }

    pub fn brief<T: Into<ErrType>>(err: T) -> Self {
        Self {
            ty: err.into(),
            span: None,
        }
    }
}

/*

API:
ShadowErrorBuilder::new()
  .set_err(ParseErrors::UnexpectedToken(tk.ty))
  .set_span(tk.span)
  .context("parsing function")
  .add_help("(", tk.span)
  .add_diagnostic("presumed function definition because of this", fn_kw.span)
  .help("function definitions expect a parameter list")
  .build()?;

OUTPUT:

[SDW E/P] unexpected token '[' whilst parsing function
-> error occured at line 1, character 10.

1 ├─ fn int main[) {
  |  ^^         ^ - help: perhaps you meant '('?
  |  |
  |  ╚  note: presumed function definition because of this.
  ├─ help: function definitions expect a parameter list

*/

#[derive(Debug)]
pub enum ErrType {
    Lex(LexErrors),
    Parse(ParseErrors),
    Semantic(SemErrors),
}

impl std::fmt::Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lex(lexerr) => format!("{}", lexerr),
                Self::Parse(parseerr) => format!("{}", parseerr),
                Self::Semantic(semerr) => format!("{}", semerr),
            }
        )
    }
}

#[derive(Error, Debug)]
pub enum LexErrors {
    #[error("unrecognised token encountered: '{0}'")]
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
    UnexpectedTokenEncountered(LexemeTypes, LexemeTypes),
    #[error("invalid LHS in expression - found the token {0}")]
    InvalidExpressionLHS(LexemeTypes),
    #[error("and unknown postfix operator was used - {0}")]
    UnknownOperator(LexemeTypes),
}

impl From<ParseErrors> for ErrType {
    fn from(other: ParseErrors) -> ErrType {
        ErrType::Parse(other)
    }
}

use crate::parse::Type;
#[derive(Debug, Error)]
pub enum SemErrors {
    #[error("the function's return type ('{0:?}') & the type of returned value ('{1:?}') do not match")]
    MismatchedFnRetTy(Type, Type),
    #[error("return statements are only allowed inside of function definitions")]
    ReturnOutsideFn,
    #[error("the compiler is not currently in a scope. idk what might cause this actually lol")]
    CompilerNotInAScope,
    #[error("the variabled named '{0}' was not defined inside the current scope")]
    VariableNotFound(String),
}

impl From<SemErrors> for ErrType {
    fn from(other: SemErrors) -> ErrType {
        ErrType::Semantic(other)
    }
}
