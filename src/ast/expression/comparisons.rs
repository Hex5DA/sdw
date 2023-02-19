use crate::ast::{
    expression::{Expression, ExpressionTrait},
    ASTNode, OutputWrapper, PrimitiveType, SymbolTable,
};
use crate::lex::Lexeme;
use anyhow::{Result, bail};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum CompTypes {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanEqualTo,
    LessThanEqualTo,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    comp_type: CompTypes,
    lhs: Expression,
    rhs: Expression,
}

impl ASTNode for Comparison {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let idx = lexemes.iter().position(|l| matches!(l, &Lexeme::AngleLeft | &Lexeme::AngleRight | Lexeme::Equals)).unwrap();
        let lhs = Expression::new(&mut lexemes.drain(..idx).collect(), symtab)?;
        // !\=
        let ty = match lexemes.pop_front().unwrap() {
            Lexeme::AngleRight => match lexemes.front().unwrap() {
                Lexeme::Equals => {
                    lexemes.pop_front().unwrap();
                    CompTypes::GreaterThanEqualTo
                },
                _ => CompTypes::GreaterThan,
            },
            Lexeme::AngleLeft => match lexemes.front().unwrap() {
                Lexeme::Equals => {
                    lexemes.pop_front().unwrap();
                    CompTypes::LessThanEqualTo
                },
                _ => CompTypes::LessThan,
            }
            Lexeme::Equals => match lexemes.pop_front().unwrap() {
                Lexeme::Equals => CompTypes::Equal,
                _ => bail!("2 equals required for equality comparison"),
            }
            Lexeme::Bang => match lexemes.pop_front().unwrap() {
                Lexeme::Equals => CompTypes::NotEqual,
                _ => bail!("unary ops not yet supported; ! should be followed by ="),
            }
            _ => bail!("TODO: improve error handling here!"), 
        };
        let rhs = Expression::new(lexemes, symtab)?;
        Ok(Self {
            lhs,
            comp_type: ty,
            rhs,
        })
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        self.lhs.codegen(ow, symtab);
        self.rhs.codegen(ow, symtab);
        ow.appendln(format!("%{} = icmp {} {} {}, {}", self.ir(symtab), match self.comp_type {
            CompTypes::Equal => "eq",
            CompTypes::NotEqual => "ne",
            CompTypes::GreaterThan => "sgt",
            CompTypes::GreaterThanEqualTo => "sge",
            CompTypes::LessThan => "slt",
            CompTypes::LessThanEqualTo => "sle",
        }, self.lhs.evaltype(symtab).unwrap().ir_type(), self.lhs.ir(symtab), self.rhs.ir(symtab)), 1);
    }
}

impl ExpressionTrait for Comparison {
    fn evaltype(&self, _symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        Ok(PrimitiveType::Bool)
    }

    fn eval(&self, _symtab: &mut SymbolTable) -> Result<String> {
        todo!()
    }

    fn ir(&self, _symtab: &mut SymbolTable) -> String {
        "condtemp".to_string()
    }
}
