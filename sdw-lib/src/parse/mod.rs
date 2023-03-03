use crate::errors::LexErrors;
use crate::prelude::*;

mod function;
use function::Function;

mod prelude {
    pub use super::{ASTNode, ASTNodeTrait, LexemeStream, PrimitiveType};
    pub use crate::errors::ParseErrors;
    pub use crate::{eat, eat_first};
}

// TODO:
// - look at re-introducing NodeType enum
// - build up AST
// - better errors?

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
// type Block = Vec<Node<Statement>>;

// TODO: resarch how to properly represent primitive types. this'll do for now.
#[derive(Debug)]
pub enum PrimitiveType {
    Int,
}

impl PrimitiveType {
    fn from_string(other: String, span: Span) -> Result<PrimitiveType> {
        Ok(match other.as_str() {
            "int" => PrimitiveType::Int,
            _ => {
                return Err(ShadowError::from_pos(LexErrors::UnrecognisedType(other), span));
            }
        })
    }
}

pub trait ASTNodeTrait {
    fn new(lexemes: &mut LexemeStream) -> Result<Self>
    where
        Self: Sized;
    fn constructed_from(&self) -> LexemeStream;
    fn span(&self) -> Span;
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ASTNode<K: ASTNodeTrait> {
    span: Span,
    constructed_from: LexemeStream,
    ty: K,
}

impl<K: ASTNodeTrait> ASTNode<K> {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let inner = <K as ASTNodeTrait>::new(lexemes)?;
        Ok(Self {
            span: inner.span(),
            constructed_from: inner.constructed_from(),
            ty: inner,
        })
    }
}

/// add eating of tokens, verifying their type and returning the values
#[macro_export]
macro_rules! eat {
    ( $variant:pat, $then:expr, $lexemes:expr, $constructed_from:expr ) => {{
        let lexeme = ($lexemes).pop_front().ok_or_else(|| {
            ShadowError::from_pos(ParseErrors::TokenStackEmpty, $constructed_from.last().unwrap().span)
        })?;
        let inner = match lexeme.ty {
            $variant => $then,
            _ => {
                return Err(ShadowError::from_pos(
                    // TODO: lexeme type pretty printing ("'fn' keyword", "identifier" ect.)
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
        assert!(
            matches!(lexeme.ty, $variant),
            "parsing a/an {}, the required intial token ({}) was not present. developer issue!",
            $node_type,
            stringify!($variant)
        );
        $constructed_from.push(lexeme);
    }};
}

pub fn parse(mut lexemes: LexemeStream) -> Result<ASTNode<Function>> {
    ASTNode::<Function>::new(&mut lexemes)
}
