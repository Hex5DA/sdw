use super::{ir::OutputWrapper, statement::Block, ASTNode, PrimitiveType, SymbolTable};
use crate::consume;
use crate::lex::{Keyword, Lexeme};
use anyhow::{bail, Result};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct Parameter {
    pub name: String,
    pub pm_type: PrimitiveType,
}

impl ASTNode for Parameter {
    fn new(lexemes: &mut VecDeque<Lexeme>, _symtab: &mut SymbolTable) -> Result<Self> {
        let mut node = Self::default();

        consume!(Lexeme::Idn(pmt) in lexemes => {
            node.pm_type = PrimitiveType::from_str(pmt)?;
        })?;
        consume!(Lexeme::Idn(nm) in lexemes => {
            node.name = nm;
        })?;

        Ok(node)
    }

    fn codegen(&self, _ow: &mut OutputWrapper, _symtab: &mut SymbolTable) {
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct Function {
    pub name: String,
    pub body: Block,
    pub return_type: PrimitiveType,
    pub params: Vec<Parameter>,
}

impl ASTNode for Function {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let mut node = Function::default();

        consume!(Lexeme::Keyword(Keyword::Fn) in lexemes)?;
        consume!(Lexeme::Idn(tp) in lexemes => {
            node.return_type = PrimitiveType::from_str(tp)?;
        })?;
        consume!(Lexeme::Idn(nm) in lexemes => {
            node.name = nm;
        })?;
        consume!(Lexeme::OpenParen in lexemes)?;

        if !matches!(lexemes.front(), Some(Lexeme::CloseParen)) {
            while !lexemes.is_empty() {
                node.params.push(Parameter::new(lexemes, symtab)?);
                match lexemes.front() {
                    Some(Lexeme::Delimiter) => {
                        consume!(Lexeme::Delimiter in lexemes)?;
                    }
                    _ => break,
                }
            }
        }

        consume!(Lexeme::CloseParen in lexemes)?;
        node.body = Block::new(lexemes, symtab)?;
        Ok(node)
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        ow.appendln(
            format!(
                "define {} @{}({}) {{",
                self.return_type.ir_type(),
                self.name,
                self.params
                    .iter()
                    .map(|pm| format!("{} %{}", pm.pm_type.ir_type(), pm.name))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            0,
        );
        self.body.codegen(ow, symtab);
        ow.appendln("}".to_string(), 0);
    }
}
