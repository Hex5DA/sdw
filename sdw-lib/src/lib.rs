pub mod errors;
pub mod lex;
pub mod parse;
pub mod sem;

mod common {
    type PosInt = u64;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Span {
        pub line: PosInt,
        pub column: PosInt,
        pub end_col: PosInt,
        pub end_line: PosInt,
    }

    impl Span {
        pub fn from_to(from: Self, to: Self) -> Self {
            Span {
                line: from.line,
                column: from.column,
                end_col: to.column,
                end_line: to.end_line,
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

pub mod consumer {
    use crate::sem::{SemBlock, SemNode};
    pub type Block = SemBlock;
    pub type Node = SemNode;

    pub mod prelude {
        pub use crate::parse::{Type, expr::Expression};
        pub use crate::sem::SemExpression;
        pub use super::*;
    }
}

