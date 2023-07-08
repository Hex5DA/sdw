pub mod lexer;

pub mod common {
    pub type SpanInt = u64;

    /// (sline, eline]; (scol, ecol]
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

        pub fn blank() -> Span {
            Self { sline: 0, eline: 0, scol: 0, ecol: 0 }
        }
    }

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
}
