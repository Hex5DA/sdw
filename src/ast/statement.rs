use super::{
    expression::{new_expr, Expression},
    function::Function,
    ir::OutputWrapper,
    variables::Assignment,
    ASTNode, PrimitiveType, SymbolTable,
};
use crate::consume; // eh.. crate root :/
use crate::lex::{Keyword, Lexeme};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Statement {
    Return(Option<Box<dyn Expression>>),
    Function(Function),
    VariableDeclaration(Assignment),
}

impl ASTNode for Statement {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        Ok(match lexemes.front().context("Unexpected EOF")? {
            Lexeme::Keyword(Keyword::Fn) => Self::Function(Function::new(lexemes, symtab)?),
            Lexeme::Keyword(Keyword::Return) => {
                consume!(Lexeme::Keyword(Keyword::Return) in lexemes)?;
                let expr = if matches!(lexemes.front().context("Unexpected EOF")?, Lexeme::Newline)
                {
                    None
                } else {
                    Some(new_expr(lexemes, symtab)?)
                };
                consume!(Lexeme::Newline in lexemes)?;
                Self::Return(expr)
            }
            Lexeme::Keyword(Keyword::Variable) | Lexeme::Keyword(Keyword::Modifier(_)) => {
                Self::VariableDeclaration(Assignment::new(lexemes, symtab)?)
            }
            unexpected => todo!(
                "token encountered: {:?}; all tokens\n{:?}",
                unexpected,
                lexemes
            ),
        })
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        let stmt = match self {
            Statement::Return(inner) => format!(
                "ret {} {}",
                if let Some(expr) = inner {
                    expr.evaltype(symtab).unwrap()
                } else {
                    PrimitiveType::Void
                }
                .ir_type(),
                match inner {
                    Some(expr) => {
                        expr.codegen(ow, symtab);
                        expr.ir(symtab)
                    }
                    None => "".to_string(),
                },
            ),

            Statement::Function(func) => {
                func.codegen(ow, symtab);
                "".to_string()
            }
            Statement::VariableDeclaration(ass /* :smirk: */) => {
                ass.codegen(ow, symtab);
                "".to_string()
            }
        };
        ow.appendln(stmt, 1);
    }
}

#[derive(Debug, Default)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

impl ASTNode for Block {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let mut node = Self::default();

        consume!(Lexeme::OpenBrace in lexemes)?;
        while !lexemes.is_empty() {
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
            node.stmts.push(Statement::new(lexemes, symtab)?);
        }
        consume!(Lexeme::CloseBrace in lexemes)?;

        Ok(node)
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        for node in &self.stmts {
            node.codegen(ow, symtab);
        }
    }
}

#[derive(Debug, Default)]
pub struct Root {
    pub stmts: Vec<Statement>,
}

impl ASTNode for Root {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let mut node = Self::default();

        while !lexemes.is_empty() {
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
            node.stmts.push(Statement::new(lexemes, symtab)?);
        }

        Ok(node)
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        for node in &self.stmts {
            node.codegen(ow, symtab);
        }
    }
}
