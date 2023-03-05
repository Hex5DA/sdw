use crate::errors::LexErrors;
use crate::prelude::*;

pub mod statements;
use statements::Block;
pub mod function;
pub mod ret;

pub mod prelude {
    pub use super::{statements::Block, ASTNode, ASTNodeTrait, LexemeStream, PrimitiveType};
    pub use crate::errors::ParseErrors;
    pub use crate::{eat, eat_first};
}

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
pub type Root = Block;

// TODO: resarch how to properly represent primitive types. this'll do for now.
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Void,
    Int,
}

impl PrimitiveType {
    fn from_string(other: String, span: Span) -> Result<PrimitiveType> {
        Ok(match other.as_str() {
            "int" => PrimitiveType::Int,
            "void" => PrimitiveType::Void,
            _ => {
                return Err(ShadowError::from_pos(LexErrors::UnrecognisedType(other), span));
            }
        })
    }

    pub fn ir_type(&self) -> &str {
        match self {
            PrimitiveType::Void => "void",
            PrimitiveType::Int => "i64",
        }
    }
}

pub trait ASTNodeTrait: Clone + std::fmt::Debug {
    fn new(lexemes: &mut LexemeStream) -> Result<Self>
    where
        Self: Sized;
    fn span(&self) -> Span;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ASTNode<K: ASTNodeTrait> {
    span: Span,
    pub ty: K,
}

impl<K: ASTNodeTrait + Clone> ASTNode<K> {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let inner = <K as ASTNodeTrait>::new(lexemes)?;
        Ok(Self {
            span: inner.span(),
            ty: inner,
        })
    }
}

/// add eating of tokens, verifying their type and returning the values
#[macro_export]
macro_rules! eat {
    ( $variant:pat, $then:expr, $lexemes:expr, $constructed_from:expr ) => {{
        let lexeme = $lexemes.pop_front().ok_or_else(|| {
            ShadowError::from_pos(ParseErrors::TokenStackEmpty, $constructed_from.last().unwrap_or_else(|| {
                panic!("whilst parsing, both the lexeme stack and the given 'constructed_from' vector were empty! DBG:\nlexemes: {:?}\nc.f: {:?}", $lexemes, $constructed_from);
            }).span)
        })?;
        let inner = match lexeme.ty {
            $variant => $then,
            _ => {
                return Err(ShadowError::from_pos(
                    ParseErrors::UnexpectedTokenEncountered(
                        format!("{}", lexeme.ty),
                        format!("pattern: {}", stringify!($variant)),
                    ),
                    lexeme.span,
                ));
            }
        };
        $constructed_from.push(lexeme);
        inner
    }};
    ( $variant:pat, $lexemes:expr, $constructed_from:expr ) => {
        eat!($variant, {}, $lexemes, $constructed_from);
    };
}

/// the first lexeme on the stack being passed to a parsing function should always
/// match the expected value, so we don't need graceful errors & can just .expect it.
/// this also runs under the assumption that we don't need the first token, because it'll normally
/// be, eg. `Fn`, `OpenBrace, `If` ect. if i do at some point, that's a problem for future me.
pub const LEXEMESTREAM_PREVIEW: usize = 5;
#[macro_export]
macro_rules! eat_first {
    ( $variant:pat, $lexemes:expr, $constructed_from:expr, $node_type:literal ) => {{
        let lexeme = $lexemes.pop_front().expect(
            format!(
                "the lexeme stack is empty whilst parsing a/an {}. developer issue!",
                $node_type
            )
            .as_str(),
        );
        use $crate::parse::LEXEMESTREAM_PREVIEW;
        assert!(
            matches!(lexeme.ty, $variant),
            "parsing a/an {}, the required intial token ({}) was not present. developer issue! next {} tokens:\n{:#?}",
            $node_type,
            stringify!($variant),
            LEXEMESTREAM_PREVIEW,
            $lexemes
                .iter()
                .take(LEXEMESTREAM_PREVIEW)
                .map(|lexeme| lexeme.ty.clone())
                .collect::<Vec<LexemeTypes>>()
        );
        $constructed_from.push(lexeme);
    }};
}

pub fn parse(mut lexemes: LexemeStream) -> Result<ASTNode<Root>> {
    let root = Root::root(&mut lexemes)?;
    Ok(ASTNode::<Root> {
        span: root.span(),
        ty: root,
    })
}
