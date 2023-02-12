use super::{ir::OutputWrapper, ASTNode, PrimitiveType, SymbolTable};
use crate::{
    consume,
    lex::{Lexeme, Literal},
};
use anyhow::{bail, Context, Result};
use dyn_clonable::clonable;
use std::collections::VecDeque;

mod mathops;
pub use mathops::*;
mod variable;
pub use variable::*;

#[derive(Debug, Clone)]
pub struct Expression {
    inner: Box<dyn ExpressionTrait>,
}

impl ASTNode for Expression {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        assert!(!lexemes.is_empty());
        let expr = match lexemes.get(1) {
            Some(Lexeme::Newline) | None => {
                match lexemes
                    .front()
                    .context("Unexpected EOF whilst parsing expression")?
                {
                    Lexeme::Literal(_) => {
                        Box::new(Literal::new(lexemes, symtab)?) as Box<dyn ExpressionTrait>
                    }
                    Lexeme::Idn(_) => {
                        Box::new(Variable::new(lexemes, symtab)?) as Box<dyn ExpressionTrait>
                    }
                    unexpected => bail!("Could not construct an expression from {unexpected:?}"),
                }
            }
            Some(next) => match next {
                Lexeme::Addition => {
                    Box::new(Addition::new(lexemes, symtab)?) as Box<dyn ExpressionTrait>
                }
                Lexeme::Subtraction => {
                    Box::new(Subtraction::new(lexemes, symtab)?) as Box<dyn ExpressionTrait>
                }
                Lexeme::Multiplication => {
                    Box::new(Multiplication::new(lexemes, symtab)?) as Box<dyn ExpressionTrait>
                }
                Lexeme::Division => {
                    Box::new(Division::new(lexemes, symtab)?) as Box<dyn ExpressionTrait>
                }
                _ => bail!(
                    "Whilst parsing an expression, an unexpected token was encountered: {:?}",
                    next
                ),
            },
        };

        Ok(Self { inner: expr })
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        self.inner.codegen(ow, symtab);
    }
}

impl ExpressionTrait for Expression {
    fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        self.inner.evaltype(symtab)
    }
    fn eval(&self, symtab: &mut SymbolTable) -> Result<String> {
        self.inner.eval(symtab)
    }
    fn ir(&self, symtab: &mut SymbolTable) -> String {
        self.inner.ir(symtab)
    }
}

#[clonable]
pub trait ExpressionTrait: Clone + std::fmt::Debug + ASTNode {
    fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType>;
    fn eval(&self, symtab: &mut SymbolTable) -> Result<String>;
    fn ir(&self, symtab: &mut SymbolTable) -> String {
        self.eval(symtab).unwrap()
    }
}

impl ASTNode for Literal {
    fn new(lexemes: &mut VecDeque<Lexeme>, _symtab: &mut SymbolTable) -> Result<Self> {
        let node: Self;
        consume!(Lexeme::Literal(lit) in lexemes => node = lit)?;
        Ok(node)
    }

    fn codegen(&self, _ow: &mut OutputWrapper, _symtab: &mut SymbolTable) {}
}

impl ExpressionTrait for Literal {
    fn evaltype(&self, _symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        Ok(match self {
            Literal::Integer(_) => PrimitiveType::Int,
        })
    }

    fn eval(&self, _symtab: &mut SymbolTable) -> Result<String> {
        Ok(match self {
            Literal::Integer(int) => *int,
        }
        .to_string())
    }
}
