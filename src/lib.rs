pub mod errors;
pub mod lex;
pub mod common;

pub mod prelude {
    use super::*;
    pub use errors::{Result, ShadowError};
    pub use lex::{Keywords, Lexeme, LexemesTypes, Literal};
    pub use common::PosInfo;
}
