use crate::parse::prelude::*;
use crate::prelude::*;

use crate::parse::{function::Function, ret::Return};

#[derive(Debug, Clone)]
pub enum StatementTypes {
    Return(ASTNode<Return>),
    FunctionDef(ASTNode<Function>),
}

impl StatementTypes {
    fn span(&self) -> Span {
        match self {
            Self::Return(r) => r.span,
            Self::FunctionDef(fd) => fd.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<StatementTypes>,
    span: Span,
}

impl ASTNodeTrait for Block {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let mut constructed_from = Vec::new();
        // TODO: this should maybe not be a dev. error?
        eat_first!(LexemeTypes::OpenBrace, lexemes, constructed_from, "block");
        let retval = Self::root(lexemes);
        eat!(LexemeTypes::CloseBrace, lexemes, constructed_from);
        retval
    }

    fn span(&self) -> Span {
        self.span
    }
}

impl Block {
    pub fn root(lexemes: &mut LexemeStream) -> Result<Self> {
        use StatementTypes as ST;

        let mut statements = Vec::new();
        loop {
            // unwrap should be improved, but i don't want to
            // newtype VecDeque // Vec yet (just too lazy lol)
            statements.push(match lexemes.front() {
                Some(lexeme) => match lexeme.ty {
                    LexemeTypes::Keyword(kw) => match kw {
                        Keywords::Fn => ST::FunctionDef(ASTNode::<Function>::new(lexemes)?),
                        Keywords::Return => ST::Return(ASTNode::<Return>::new(lexemes)?),
                    },
                    _ => break,
                },
                None => break,
            });
        }

        let span = if !statements.is_empty() {
            Span::from_to(statements.first().unwrap().span(), statements.last().unwrap().span())
        } else {
            // dit. above
            lexemes.front().unwrap().span
        };

        Ok(Self { statements, span })
    }
}
