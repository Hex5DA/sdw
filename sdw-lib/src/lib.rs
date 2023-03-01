pub mod errors;
pub mod lex;
pub mod parse;

mod common {
    type PosInt = u64;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Span {
        pub line: PosInt,
        pub column: PosInt,
        pub length: PosInt,
    }

    impl Span {
        pub fn from_to(from: Self, to: Self) -> Self {
            Span {
                line: from.line,
                column: from.column,
                length: to.column - from.column,
            }
        }
    }
}

pub mod prelude {
    use super::*;
    pub use common::Span;
    pub use errors::{Result, ShadowError};
    pub use lex::{Keywords, Lexeme, LexemeTypes, Literal};
}
