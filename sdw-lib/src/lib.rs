pub mod errors;
pub mod lex;
pub mod parse;
pub mod sem;

mod common {
    use crate::errors::{LexErrors, Result, ShadowError};

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

    #[derive(Debug, Clone)]
    pub struct Spanned<T> {
        pub inner: T,
        pub span: Span,
    }

    impl<T> Spanned<T> {
        pub fn new(span: Span, inner: T) -> Self {
            Self { span, inner }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Type {
        Void,
        Bool,
        Int,
    }

    impl Type {
        pub fn from_string(other: String, span: Span) -> Result<Type> {
            Ok(match other.as_str() {
                "int" => Type::Int,
                "void" => Type::Void,
                "bool" => Type::Bool,
                _ => return Err(ShadowError::from_pos(LexErrors::UnrecognisedType(other), span)),
            })
        }
    }
}

pub mod mangle {
    use lazy_static::lazy_static;
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Mutex,
        },
    };

    lazy_static! {
        static ref FN_MT: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
        static ref VA_MT: Mutex<HashMap<String, AtomicUsize>> = Mutex::new(HashMap::new());
    }

    pub fn mangle_fn(name: String) -> String {
        FN_MT.lock().unwrap().insert(name.clone(), name.clone());
        name
    }

    /*
        fn int main() {
            let v = 5;
            return v;
        }

        define i64 @main() {
          ; %0 is v
          %v0 = alloca i64
          store i64 5, ptr %v0
          ; %1 is v derefenced
          %v1 = load i64, ptr %v0
          ret i64 %v1
        }

        - when creating v we make a link between "v" and "v0"
        - when storing 5 in v, we look up "v" and get "v0"
        - when we want to get the value of v, we look up "v" and find
        it has a highest "mangle number" of 0, so add 1 to get "v1"
        we don't need to create a link for this, because "v1" will only be used
        in the context in which it is declared, so can be returned by the generative function
        - in ret, the generative function, as mentioned prior, returns the tag "v1" (which is then used in translation)
    */

    pub fn mangle_va(name: String) -> String {
        let mut lock = VA_MT.lock().unwrap();
        let mangle_count = if let Some(at) = lock.get(&name) {
            at.fetch_add(1, /* idk what this arg does lol */ Ordering::SeqCst) + 1
        } else {
            lock.insert(name.clone(), AtomicUsize::new(0));
            0
        };
        format!("{}.{}", name, mangle_count)
    }

    // not sure if this is dodging around the problem or
    // a good solution - so treat this as temporary
    pub fn mangle_va_base(name: String) -> String {
        format!("{}.0", name)
    }
}

pub mod prelude {
    use super::*;
    pub use common::{Span, Spanned};
    pub use errors::{Result, ShadowError};
    pub use lex::{Keywords, Lexeme, LexemeTypes, Literals};
}

pub mod consumer {
    use crate::sem::{AbstractBlock, AbstractExpression, AbstractExpressionType, AbstractNode, AbstractNodeType};

    pub type Block = AbstractBlock;
    pub type Node = AbstractNode;
    pub type NodeType = AbstractNodeType;
    pub type Expression = AbstractExpression;
    pub type ExpressionType = AbstractExpressionType;

    pub mod prelude {
        pub use super::*;
        pub use crate::common::Type;
    }
}
