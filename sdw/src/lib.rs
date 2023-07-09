pub mod lexer;
pub mod errors;

pub mod common {
    pub type SpanInt = u64;

    // TODO: inclusive or exclusive??
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

    #[derive(Debug)]
    pub struct Spanned<T> {
        pub spanned: T,
        pub span: Span,
    }

    impl<T> Spanned<T> {
        pub fn new(spanned: T, span: Span) -> Spanned<T> {
            Self {
                spanned, span
            }
        }
    }
}

pub mod prelude {
    pub use crate::common::*;
    pub use crate::errors::{Result, ErrType, LexErrors, SdwErr};
}