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

#[derive(Debug, Clone)]
pub struct Subtraction(Expression, Expression);
impl ASTNode for Subtraction {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let idx = lexemes.iter().position(|l| l == &Lexeme::Subtraction).unwrap();
        let rhs = Expression::new(&mut lexemes.drain(..idx).collect(), symtab)?;
        consume!(Lexeme::Subtraction in lexemes)?;
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
                "%{} = sub {} {}, {}",
                self.ir(symtab),
                self.0.evaltype(symtab).unwrap().ir_type(),
                self.0.eval(symtab).unwrap(),
                self.1.eval(symtab).unwrap()
            ),
            1,
        );
    }
}

impl ExpressionTrait for Subtraction {
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
        "subtemp".to_string()
    }
}


#[derive(Debug, Clone)]
pub struct Multiplication(Expression, Expression);
impl ASTNode for Multiplication {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let idx = lexemes.iter().position(|l| l == &Lexeme::Multiplication).unwrap();
        let rhs = Expression::new(&mut lexemes.drain(..idx).collect(), symtab)?;
        consume!(Lexeme::Multiplication in lexemes)?;
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
                "%{} = mul {} {}, {}",
                self.ir(symtab),
                self.0.evaltype(symtab).unwrap().ir_type(),
                self.0.eval(symtab).unwrap(),
                self.1.eval(symtab).unwrap()
            ),
            1,
        );
    }
}

impl ExpressionTrait for Multiplication {
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
        "multemp".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Division(Expression, Expression);
impl ASTNode for Division {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let idx = lexemes.iter().position(|l| l == &Lexeme::Division).unwrap();
        let rhs = Expression::new(&mut lexemes.drain(..idx).collect(), symtab)?;
        consume!(Lexeme::Division in lexemes)?;
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
                "%{} = div {} {}, {}",
                self.ir(symtab),
                self.0.evaltype(symtab).unwrap().ir_type(),
                self.0.eval(symtab).unwrap(),
                self.1.eval(symtab).unwrap()
            ),
            1,
        );
    }
}

impl ExpressionTrait for Division {
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
        "divtemp".to_string()
    }
}
