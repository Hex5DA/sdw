use crate::lex::{Lexeme, Keyword};
use crate::ast::{
    SymbolTable,
    ASTNode,
    OutputWrapper,
    statement::{Block, Statement},
    expression::Expression,
};
use crate::consume;
use std::collections::VecDeque;
use anyhow::{Result, Context, bail};

// conditionals always start with an if
// they optionally have an unlimited number of else ifs
// they optionally end with an else

// else ifs can be represented as conditonals
// thus recursive defintion: else ifs are a vector of conditonals
// optional else conditional

#[derive(Debug)]
pub struct Conditional {
    cond: ConditionalItem,
    elifs: Option<Vec<ConditionalItem>>,
    else_block: Option<Block>,
}

impl ASTNode for Conditional {
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self> {
        consume!(Lexeme::Keyword(Keyword::If) in lexemes)?;
        let cond = ConditionalItem::new(lexemes, symtab).context("malformed if expression, maybe")?;
        
        let mut elifs = Vec::new();
        while let Some(conditem) = ConditionalItem::new(lexemes, symtab) {
            elifs.push(conditem);
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
        }

        let mut else_block = None;
        if let Some(Lexeme::Keyword(Keyword::Else)) = lexemes.front() {
            lexemes.pop_front().unwrap();
            if let Some(Lexeme::OpenBrace) = lexemes.front() {
                else_block = Some(Block::new(lexemes, symtab)?);
            } else {
                else_block = Some(Block::from_statements(vec![Statement::new(lexemes, symtab)?]));
            }
        }

        Ok(Self {
            cond,
            elifs: if !elifs.is_empty() { Some(elifs) } else { None },
            else_block,
        })
    }

    fn codegen(&self, _ow: &mut OutputWrapper, _symtab: &mut SymbolTable) {
        todo!()
    }
}

#[derive(Debug)]
pub struct ConditionalItem {
    expr: Expression,
    body: Block,
}

impl ConditionalItem {
    // TODO: return Result<Option<Self>>, error handling
    fn new(lexemes: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Option<Self> {
        if lexemes.front() == Some(&Lexeme::Keyword(Keyword::Else)) {
            if lexemes.get(1) != Some(&Lexeme::Keyword(Keyword::If)) {
                return None;
            }
            // TODO: replace with consume!()
            lexemes.pop_front().unwrap();
            lexemes.pop_front().unwrap();
        }    

        let expir: &mut VecDeque<Lexeme> = &mut lexemes.iter().cloned().collect();
        let _ = Expression::new(expir, symtab);
        // TODO: verify there is a statement, this sucks
        let single_stmt = *expir.front()? != Lexeme::OpenBrace;
        Some(if single_stmt {
            assert_eq!(lexemes.pop_front(), Some(Lexeme::OpenParen), "parenthese expected in shorthand notation");
            let expr = Expression::new(lexemes, symtab).ok()?;
            assert_eq!(lexemes.pop_front(), Some(Lexeme::CloseParen), "parenthese expected in shorthand notation");
            let body = Block::from_statements(vec![Statement::new(lexemes, symtab).ok()?]);
            Self { expr, body }
        } else {
            let expr = Expression::new(lexemes, symtab).ok()?;
            let body = Block::new(lexemes, symtab).ok()?;
            Self { expr, body }
        })
    }
}

