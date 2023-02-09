use super::{
    expression::{new_expr, Expression},
    ir::OutputWrapper,
    ASTNode, PrimitiveType, SymbolTable, Var,
};
use crate::consume;
use crate::lex::{Keyword, Lexeme, Modifier};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct Assignment {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub value: Option<Box<dyn Expression>>,
    pub vtype: Option<PrimitiveType>,
}

impl ASTNode for Assignment {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let mut node = Self::default();
        while let Lexeme::Keyword(Keyword::Modifier(modi)) =
            lexemes.pop_front().context("Unexpcted EOF")?
        {
            node.modifiers.push(modi);
        }

        consume!(Lexeme::Idn(nm) in lexemes => {
            node.name = nm;
        })?;

        if let Lexeme::Keyword(Keyword::Coercion) = lexemes.front().context("Unexpected EOF")? {
            consume!(Lexeme::Keyword(Keyword::Coercion) in lexemes)?;
            consume!(Lexeme::Idn(ty) in lexemes => {
                node.vtype = Some(PrimitiveType::from_str(ty)?);
            })?;
        }

        node.value = match lexemes.pop_front().context("Unexpected EOF")? {
            Lexeme::Newline => None,
            Lexeme::Assignment => {
                let expr = new_expr(lexemes, symtab)?;
                consume!(Lexeme::Newline in lexemes)?;
                Some(expr)
            }
            _ => bail!("Expected variable intialiser or newline."),
        };

        // TODO: support implicit declarations throughvariable usage
        if node.value.is_none() && node.vtype.is_none() {
            bail!("Either a specified type or initaliser must be present.");
        }

        symtab.insert(
            node.name.clone(),
            Var {
                name: node.name.clone(),
                vtype: node.vtype,
                value: node.value.clone(),
            },
        );

        Ok(node)
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        let ty = if self.vtype.is_none() {
            // None first because borrow checker :/
            self.value.as_ref().unwrap().evaltype(symtab).unwrap()
        } else {
            self.vtype.unwrap()
        };

        ow.appendln(format!("%{} = alloca {}", self.name, ty.ir_type()), 1);
        if let Some(val) = &self.value {
            val.codegen(ow, symtab);
            ow.appendln(
                format!(
                    "store {} {}, ptr %{}",
                    ty.ir_type(),
                    match val.eval(symtab) {
                        Ok(v) => v.to_string(),
                        Err(v) => format!("%{}", v.to_string()),
                    },
                    self.name
                ),
                1,
            );
        }
    }
}
