pub mod common;
pub mod errors;
pub mod lex;

pub mod prelude {
    use super::*;
    pub use common::PosInfo;
    pub use errors::{Result, ShadowError};
    pub use lex::{Keywords, Lexeme, LexemeTypes, Literal};
}
