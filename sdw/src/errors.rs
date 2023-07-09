use owo_colors::OwoColorize;
use crate::common::Span;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SdwErr>;

#[derive(Debug)]
pub struct SdwErr {
    pub ty: ErrType,
    pub span: Span,
}

fn repeat_char(ch: char, len: usize) -> String {
    std::iter::repeat(ch).take(len).collect::<String>()
}

impl SdwErr {
    fn header(&self) {
        eprint!(
            "{} ",
            format!(
                "[SDW E/{}]",
                match self.ty {
                    ErrType::Lex(_) => "L",
                },
            )
            .red()
        );
        eprintln!("{}", self.ty);
        eprintln!(
            "{} error occurred at {}, {}.",
            "->".blue(),
            ("line ".to_owned() + &(self.span.sline + 1).to_string()).blue(),
            ("character ".to_owned() + &(self.span.scol + 1).to_string()).blue()
        );
    }

    fn body(&self, raw: &str) {
        println!("{:?}", self.span);
        let lines = raw.split('\n').collect::<Vec<&str>>();
        if self.span.sline > 1 {
            eprintln!("[ .. ]")
        };
        // idk if this works
        for line in self.span.sline..=self.span.eline {
            eprintln!(
                "{}",
                lines
                    .get(line as usize)
                    .expect("an error was reported on a line that does not exist.. somehow")
            );
            eprintln!(
                "{}{} {}",
                repeat_char(' ', self.span.scol as usize - 1),
                repeat_char('^', (self.span.ecol - self.span.scol + 1) as usize).red(),
                "- error occured here!".red()
            );
        }
        if self.span.sline as usize == lines.len() {
            eprintln!("[ .. ]")
        };
    }

    pub fn print(&self, raw: &str) {
        self.header();
        self.body(raw);
    }

    pub fn from_pos<T: Into<ErrType>>(err: T, span: Span) -> Self {
        Self { ty: err.into(), span }
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
}

impl std::fmt::Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lex(lexerr) => format!("{}", lexerr),
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

