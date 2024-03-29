use crate::common::Span;
use owo_colors::OwoColorize;
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
                    ErrType::Parse(_) => "P",
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
        let lines = raw.split('\n').collect::<Vec<&str>>();
        if self.span.sline > 0 {
            eprintln!("{}", "[ .. ]".bright_green());
        };

        for idx in self.span.sline..self.span.eline {
            let line = lines.get(idx as usize).unwrap();

            // ew
            let scol = if idx == self.span.sline {
                self.span.scol as usize
            } else {
                1
            };
            let ecol = if idx + 1 == self.span.eline {
                self.span.ecol as usize
            } else {
                line.len() + 1
            };
            let notice = if idx + 1 == self.span.eline {
                " - error occured here"
            } else {
                ""
            };

            eprintln!("{}", line);
            eprintln!(
                "{}{}{}",
                repeat_char(' ', scol - 1),
                repeat_char('^', ecol - scol).red(),
                notice.red()
            );
        }

        if self.span.eline as usize + 1 != lines.len() {
            eprintln!("{}", "[ .. ]".bright_green());
        };
    }

    pub fn print(&self, raw: &str) {
        self.header();
        self.body(raw);
    }

    pub fn from_pos<T: Into<ErrType>>(err: T, span: Span) -> Self {
        Self {
            ty: err.into(),
            span,
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
}

impl std::fmt::Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lex(err) => format!("{}", err),
                Self::Parse(err) => format!("{}", err),
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
    #[error("expected a type")]
    ExpectedType,
    #[error("expected a function return type")]
    MissingFnReturnType,
    #[error("no function name provided")]
    MissingFnIdn,
    #[error("the function argument list was not closed (expected a `)`)")]
    FnArgListNotClosed,
    #[error("expected function stubs' parameters' types")]
    ExpectedArgType,
    #[error("expected function parameters' identifiers")]
    ExpectedArgIdn,
    #[error("function stub arguments did not have a comma delimeter. help: have you given an argument a name?")]
    StubNoArgDel,

    #[error("statements should end with a semicolon (`;`)")]
    StmtsEndWithSemi,
    #[error("block not opened - expected an opening brace (`{{`)")]
    BlockNotOpened,
    #[error("block not closed - expected a closing brace (`}}`)")]
    BlockNotClosed,
    #[error("label was not given a name")]
    LabelName,
    #[error("goto label should start with an `@`")]
    LabelNamePrefixedAt,
    #[error("goto should be provided with a label to jump to")]
    GotoNeedLabel,
    #[error("return has no expression")]
    NoReturnExpr,
    #[error("variable not given a name")]
    NoVarName,
    #[error("expected an equals following `let` declaration")]
    ExpectedEquals,
    #[error("expected an initialiser expression in a `let` declaration")]
    NoLetInitialiser,
    #[error("type declaration expects a name")]
    NoTypeDecName,
    #[error("struct member not provided with a name")]
    NoMemberName,
    #[error("missing a type in a function pointer type declration")]
    FnPtrTyNoType,
    #[error("malformed function pointer type arrow (expected `->`)")]
    FnPtrTyArrow,
    #[error("expected a return type in function pointer type")]
    ExpectedFnPtrReturnTy,
    #[error("type bound not recognised")]
    InvalidBound,

    #[error("subexpr not closed - expected a closing paren (`)`)")]
    SubExprNotClosed,

    #[error("reached the end of the token stack; {0}")]
    TkStackEmpty(Box<ParseErrors>),
    #[error("expected an identifier")]
    ExpectedIdn,
    #[error("could not determine the next location in the source file")]
    NoMoreSpans, // TODO: avoidable? better diagnostic?
    #[error("expected function body or semicolon (`;`)")]
    NoFnBodyStub,
    #[error("no function args list provided")]
    NoFnArgs,
}

impl From<ParseErrors> for ErrType {
    fn from(other: ParseErrors) -> ErrType {
        ErrType::Parse(other)
    }
}
