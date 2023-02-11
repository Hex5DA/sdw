use crate::ast::{
    expression::{Expression, ExpressionTrait},
    ir::OutputWrapper,
    ASTNode, PrimitiveType, SymbolTable,
};
use crate::{
    consume,
    lex::{Lexeme},
};
use anyhow::{bail, Result};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Addition(Expression, Expression);
impl ASTNode for Addition {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let idx = lexemes.iter().position(|l| l == &Lexeme::Addition).unwrap();
        let rhs = Expression::new(&mut lexemes.drain(..idx).collect(), symtab)?;
        consume!(Lexeme::Addition in lexemes)?;
        let lhs = Expression::new(lexemes, symtab)?;
        Ok(Self(rhs, lhs))
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        assert_eq!(
            self.0.evaltype(symtab).unwrap(),
            self.1.evaltype(symtab).unwrap()
        );
        ow.appendln(
            format!(
                "%{} = add {} {}, {}",
                self.ir(symtab),
                self.0.evaltype(symtab).unwrap().ir_type(),
                self.0.eval(symtab).unwrap(),
                self.1.eval(symtab).unwrap()
            ),
            1,
        );
    }
}

impl ExpressionTrait for Addition {
    fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        let lhs = self.0.evaltype(symtab)?;
        assert_eq!(lhs, self.1.evaltype(symtab)?);
        Ok(lhs)
    }

    fn eval(&self, symtab: &mut SymbolTable) -> Result<String> {
        // vv expression simplification
        // self.0.eval() + self.1.eval()
        // unreachable!()
        Ok(format!("%{}", self.ir(symtab)))
    }

    fn ir(&self, _symtab: &mut SymbolTable) -> String {
        "addtemp".to_string()
    }
}
