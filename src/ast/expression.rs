use super::{ir::OutputWrapper, ASTNode, PrimitiveType, SymbolTable};
use crate::{
    consume,
    lex::{Lexeme, Literal},
};
use anyhow::{bail, Context, Result};
use dyn_clonable::clonable;
use std::collections::VecDeque;

#[clonable]
pub trait Expression: ASTNode + Clone {
    fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType>;
    fn eval(&self, symtab: &mut SymbolTable) -> Result<i64>;
    fn ir(&self, symtab: &mut SymbolTable) -> String {
        self.eval(symtab).unwrap().to_string()
    }
}

pub fn new_expr(
    lexemes: &mut VecDeque<Lexeme>,
    symtab: &mut SymbolTable,
) -> Result<Box<dyn Expression>> {
    Ok(match lexemes
        .front()
        .context("Unexpected EOF whilst parsing expression")?
    {
        Lexeme::Literal(_) => Box::new(Literal::new(lexemes, symtab)?) as Box<dyn Expression>,
        Lexeme::Idn(_) => Box::new(Variable::new(lexemes, symtab)?) as Box<dyn Expression>,
        unexpected => bail!("Could not construct an expression from {unexpected:?}"),
    } as Box<dyn Expression>)
}

impl ASTNode for Literal {
    fn new(lexemes: &mut VecDeque<Lexeme>, _symtab: &mut SymbolTable) -> Result<Self> {
        let node: Self;
        consume!(Lexeme::Literal(lit) in lexemes => node = lit)?;
        Ok(node)
    }

    fn codegen(&self, _ow: &mut OutputWrapper, _symtab: &mut SymbolTable) {}
}

impl Expression for Literal {
    fn evaltype(&self, _symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        Ok(match self {
            Literal::Integer(_) => PrimitiveType::Int,
        })
    }

    fn eval(&self, _symtab: &mut SymbolTable) -> Result<i64> {
        Ok(match self {
            Literal::Integer(int) => *int,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Variable(String);
impl ASTNode for Variable {
    fn new(lexemes: &mut VecDeque<Lexeme>, _symtab: &mut SymbolTable) -> Result<Self> {
        let mut node = Self::default();
        consume!(Lexeme::Idn(nm) in lexemes => node.0 = nm)?;
        Ok(node)
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        ow.appendln(
            format!(
                "{} = load {}, ptr %{}",
                self.ir(symtab),
                self.evaltype(symtab).unwrap().ir_type(),
                self.0
            ),
            1,
        );
    }
}

impl Expression for Variable {
    fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        let var = symtab
            .get(&self.0)
            .context(format!("Variable {} not found in scope", self.0))?;

        Ok(if let Some(strict) = var.vtype {
            strict
        } else {
            let val = var.value.clone();
            val.as_ref()
                .unwrap()
                .evaltype(symtab)
                .context("The variable's value was another variable, not yet supported")?
        })
    }

    fn eval(&self, _symtab: &mut SymbolTable) -> Result<i64> {
        // vv constant, folding, want reference passing
        // let var = symtab.get(nm).context(format!("Variable {nm} not found in scope"))?;
        // let val = var.value.clone().context(format!("The variable {nm} has no defined value"))?;
        // val.eval(symtab)?
        unreachable!()
    }

    fn ir(&self, _symtab: &mut SymbolTable) -> String {
        format!("%{}deref", self.0)
    }
}