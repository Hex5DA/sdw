use anyhow::{bail, Context, Result};
use crate::lex::{Literal, Lexeme};
use super::{PrimitiveType, SymbolTable, ir::OutputWrapper, ASTNode};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String),
    Literal(Literal),
}

impl Expression {
    pub fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        Ok(match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(_) => PrimitiveType::Int,
            },
            Self::Variable(nm) => {
                let var = symtab
                    .get(nm)
                    .context(format!("Variable {nm} not found in scope"))?;
                // var.vtype
                //    .context(format!("The variable {nm} has no strictly defined type"))?;
                if let Some(strict) = var.vtype {
                    strict
                } else {
                    var.value
                        .clone() // ew
                        .unwrap()
                        .evaltype(symtab)
                        .context("The variable's value was another variable, not yet supported")?
                }
            }
        })
    }

    pub fn eval(&self, _symtab: &mut SymbolTable) -> Result<i64> {
        Ok(match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(inner) => *inner,
            },
            Self::Variable(_) => {
                // vv constant, folding, want reference passing
                // let var = symtab.get(nm).context(format!("Variable {nm} not found in scope"))?;
                // let val = var.value.clone().context(format!("The variable {nm} has no defined value"))?;
                // val.eval(symtab)?
                unreachable!()
            }
        })
    }

    pub fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        match self {
            Self::Variable(nm) => {
                let var = symtab
                    .get(nm)
                    .expect(format!("Variable {nm} not found in scope").as_str());

                let intname = format!("{}deref", var.name.clone());
                ow.appendln(
                    format!(
                        "%{} = load {}, ptr %{}",
                        intname,
                        self.evaltype(symtab).unwrap().ir_type(),
                        nm.clone()
                    ),
                    1,
                );
            }
            Self::Literal(_) => {}
        }
    }
}

impl ASTNode for Expression {
    fn new(lexemes: &mut VecDeque<Lexeme>, _symtab: &mut SymbolTable) -> Result<Self> {
        Ok(match lexemes.pop_front().context("Unexpected EOF")? {
            Lexeme::Literal(lit) => Self::Literal(lit),
            Lexeme::Idn(nm) => Self::Variable(nm),
            _ => bail!("Only literal expressions are supported for now!"),
        })
    }

    fn codegen(&self, _ow: &mut OutputWrapper, _symtab: &mut SymbolTable) {
        todo!()
    }
}
