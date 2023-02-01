use anyhow::{bail, Context, Result};
use crate::lex::{Lexeme, Keyword, Modifier};
use crate::consume;
use super::{Var, ASTNode, PrimitiveType, expression::Expression, SymbolTable, ir::OutputWrapper};
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct Assignment {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub value: Option<Expression>,
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
                let expr = Expression::new(lexemes, symtab)?;
                consume!(Lexeme::Newline in lexemes)?;
                Some(expr)
            }
            _ => bail!("Expected variable intialiser or newline."),
        };

        // TODO: support implicit declarations throughvariable usage
        if let None = node.value {
            if let None = node.vtype {
                // no chaining if lets yet?
                bail!("Either a specified type or initaliser must be present.");
            }
        }

        symtab.insert(
            node.name.clone(),
            Var {
                name: node.name.clone(),
                vtype: node.vtype.clone(),
                value: node.value.clone(),
            },
        );

        Ok(node)
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        let ty = if let None = self.vtype {
            // None first because borrow checker :/
            self.value.as_ref().unwrap().evaltype(symtab).unwrap()
        } else {
            self.vtype.unwrap()
        };

        ow.appendln(format!("%{} = alloca {}", self.name, ty.ir_type()), 1);
        if let Some(val) = &self.value {
            ow.appendln(
                format!(
                    "store {} {}, ptr %{}",
                    ty.ir_type(),
                    val.eval(symtab).unwrap(),
                    self.name
                ),
                1,
            );
        }
    }}
