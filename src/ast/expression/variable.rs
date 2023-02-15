use crate::{
    ast::{expression::ExpressionTrait, ir::OutputWrapper, ASTNode, PrimitiveType, SymbolTable},
    consume,
    lex::Lexeme,
};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

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

impl ExpressionTrait for Variable {
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

    fn eval(&self, _symtab: &mut SymbolTable) -> Result<String> {
        // vv constant folding, want reference passing for now
        // let var = symtab.get(nm).context(format!("Variable {nm} not found in scope"))?;
        // let val = var.value.clone().context(format!("The variable {nm} has no defined value"))?;
        // val.eval(symtab)?
        unreachable!()
    }

    fn ir(&self, _symtab: &mut SymbolTable) -> String {
        format!("%{}deref", self.0)
    }
}
