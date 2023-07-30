pub mod errors;
pub mod lexer;
pub mod parser;

pub mod common {
    use owo_colors::OwoColorize;

    #[derive(Default)]
    pub struct State {
        pub errors: Vec<crate::errors::SdwErr>,
    }

    impl State {
        pub fn new() -> Self {
            Self { errors: Vec::new() }
        }

        /// expects caller to error out.
        pub fn print_errs(&self, contents: &str, process: &str) {
            let err_text = format!(
                "{} error{}",
                self.errors.len(),
                if self.errors.len() == 1 { "" } else { "s" }
            );

            eprintln!(
                "summary: {} raised whilst {}.\n",
                err_text.red(),
                process.bright_green()
            );

            for (idx, error) in self.errors.iter().enumerate() {
                eprintln!("\n~= {} #{} =~", "error".red(), idx + 1);
                error.print(contents);
            }
        }
    }

    pub type SpanInt = u64;

    /// (sline, eline] & (scol, ecol]
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Span {
        pub sline: SpanInt,
        pub eline: SpanInt,
        pub scol: SpanInt,
        pub ecol: SpanInt,
    }

    impl Span {
        pub fn from_to(from: Span, to: Span) -> Span {
            assert!(from.sline <= to.eline);
            assert!(from.scol <= to.ecol || to.eline > from.sline);

            Span {
                sline: from.sline,
                eline: to.eline,
                scol: from.scol,
                ecol: to.ecol,
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Spanned<T> {
        pub spanned: T,
        pub span: Span,
    }

    impl<T> Spanned<T> {
        pub fn new(spanned: T, span: Span) -> Spanned<T> {
            Self { spanned, span }
        }
    }
}

pub mod prelude {
    pub use crate::common::*;
    pub use crate::errors::{ErrType, LexErrors, ParseErrors, Result, SdwErr};
    pub use crate::lexer::{Lexeme, LexemeType};
    pub use crate::parser::prelude::*;
}
