pub mod errors;
pub mod lexer;
pub mod parser;

pub mod common {
    use derivative::Derivative;

    #[derive(Default)]
    pub struct State {
        pub errors: Vec<crate::errors::SdwErr>,
    }

    impl State {
        pub fn new() -> Self {
            Self { errors: Vec::new() }
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
            assert!(from.scol <= to.ecol);

            Span {
                sline: from.sline,
                eline: to.eline,
                scol: from.scol,
                ecol: to.ecol,
            }
        }
    }

    #[derive(Derivative, Clone, Copy)]
    #[derivative(Debug)]
    pub struct Spanned<T> {
        pub spanned: T,
        #[derivative(Debug = "ignore")]
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
}
