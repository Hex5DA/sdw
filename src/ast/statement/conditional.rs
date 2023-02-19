use crate::ast::{expression::{Expression, ExpressionTrait}, statement::Block, ASTNode, OutputWrapper, SymbolTable};
use crate::consume;
use crate::lex::{Keyword, Lexeme};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Conditional {
    cond: ConditionalItem,
    elifs: Option<Vec<ConditionalItem>>,
    else_block: Option<Block>,
}

impl ASTNode for Conditional {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        let cond =
            ConditionalItem::new(lexemes, symtab)?.context("malformed if expression, maybe")?;

        let mut elifs = Vec::new();
        while let Some(conditem) = ConditionalItem::new(lexemes, symtab)? {
            elifs.push(conditem);
            // this is probably maybe possibly bad
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
        }

        let else_block = if let Lexeme::Keyword(Keyword::Else) =
            lexemes.front().context("unexpected eof")?
        {
            lexemes.pop_front();
            Some(Block::new(lexemes, symtab)?)
        } else {
            None
        };

        Ok(Self {
            cond,
            elifs: if !elifs.is_empty() { Some(elifs) } else { None },
            else_block,
        })
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        self.cond.codegen(ow, symtab);
        if let Some(elifs) = &self.elifs {
            for cnditm in elifs {
                cnditm.codegen(ow, symtab);
            }
        }
        if let Some(e) = &self.else_block {
            e.codegen(ow, symtab);
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ConditionalItem {
    expr: Expression,
    body: Block,
}

impl ConditionalItem {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Option<Self>> {
        match lexemes.front() {
            Some(&Lexeme::Keyword(Keyword::If)) => {
                consume!(Lexeme::Keyword(Keyword::If) in lexemes)?
            }
            Some(&Lexeme::Keyword(Keyword::Else)) => {
                if lexemes.get(1) != Some(&Lexeme::Keyword(Keyword::If)) {
                    return Ok(None);
                }
                consume!(Lexeme::Keyword(Keyword::Else) in lexemes)?;
                consume!(Lexeme::Keyword(Keyword::If) in lexemes)?;
            }
            _ => return Ok(None),
        }

        Ok(Some(Self {
            expr: Expression::new(lexemes, symtab)?,
            body: Block::new(lexemes, symtab)?,
        }))
    }

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        self.expr.codegen(ow, symtab);
        ow.appendln(format!("br i1 %{}, label %IfTrue, label %IfFalse", self.expr.ir(symtab)), 1);
        ow.appendln("IfTrue:".to_string(), 0);
        self.body.codegen(ow, symtab);
        ow.appendln("IfFalse:".to_string(), 0);
    }
}
