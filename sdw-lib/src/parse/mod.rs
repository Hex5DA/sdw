use crate::errors::{LexErrors, ParseErrors};
use crate::prelude::*;

pub mod expr;
use expr::*;

pub mod prelude {
    pub use super::expr::Expression;
    pub use super::{SyntaxBlock, SyntaxNode, Type};
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Void,
}

impl Type {
    fn from_string(other: String, span: Span) -> Result<Type> {
        Ok(match other.as_str() {
            "int" => Type::Int,
            "void" => Type::Void,
            _ => return Err(ShadowError::from_pos(LexErrors::UnrecognisedType(other), span)),
        })
    }
}

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
pub type SyntaxBlock = Vec<SyntaxNode>;

pub enum SyntaxNodeType {
    Function {
        name: String,
        params: Vec<(String, String)>,
        return_ty: String,
        body: SyntaxBlock,
    },
    Return {
        expr: Option<Expression>
    },
    VDec {
        init: Expression,
        name: String,
    }
}

pub struct SyntaxNode {
    ty: SyntaxNodeType,
    span: Span,
}

impl SyntaxNode {
    fn new(ty: SyntaxNodeType, span: Span) -> SyntaxNode {
        SyntaxNode { span, ty }
    }
}


#[derive(Debug)]
pub struct ParseBuffer {
    working: LexemeStream,
    lexemes: Vec<Lexeme>,
}

impl ParseBuffer {
    pub fn new(lexemes: Vec<Lexeme>) -> Self {
        assert!(!lexemes.is_empty(), "attempted to construct parser with no lexemes");
        Self {
            working: lexemes.clone().into(),
            lexemes,
        }
    }
    
    fn done(&self) -> bool {
        self.working.is_empty()
    }

    fn consume(&mut self, ty: LexemeTypes) -> Result<Lexeme> {
        let tk = self.pop()?;
        if tk.ty == ty {
            Ok(tk)
        } else {
            Err(ShadowError::from_pos(
                ParseErrors::UnexpectedTokenEncountered(tk.ty, ty),
                tk.span,
            ))
        }
    }

    fn peek(&mut self) -> Result<Lexeme> {
        match self.working.front() {
            Some(l) => Ok(l.clone()),
            None => Err(ShadowError::from_pos(
                ParseErrors::TokenStackEmpty,
                self.lexemes.last().unwrap().span,
            )),
        }
    }

    fn pop(&mut self) -> Result<Lexeme> {
        match self.working.pop_front() {
            Some(l) => Ok(l),
            None => Err(ShadowError::from_pos(
                ParseErrors::TokenStackEmpty,
                self.lexemes.last().unwrap().span,
            )),
        }
    }

    fn eat_idn(&mut self) -> Result<(String, Lexeme)> {
        let tk = self.pop()?;
        match tk.ty {
            LexemeTypes::Idn(ref s) => Ok((s.to_string(), tk)),
            _ => Err(ShadowError::from_pos(
                ParseErrors::UnexpectedTokenEncountered(tk.ty, LexemeTypes::Idn("<idn>".to_string())),
                tk.span,
            )),
        }
    }

    fn parse_fndef(&mut self) -> Result<SyntaxNode> {
        let start = self.consume(LexemeTypes::Keyword(Keywords::Fn))?.span;
        let (ty, ty_l) = self.eat_idn()?;
        let (nm, _nm_l) = self.eat_idn()?;
        self.consume(LexemeTypes::OpenParen)?;
        let mut params = Vec::new();
        while let Some(Lexeme {
            ty: LexemeTypes::Idn(_),
            ..
        }) = self.working.front()
        {
            // why does rustfmt do this :sob:
            let (ty, _ty_l) = self.eat_idn()?;
            let (nm, _nm_l) = self.eat_idn()?;
            params.push((nm, ty));
            if let Some(Lexeme {
                ty: LexemeTypes::Comma, ..
            }) = self.working.front()
            {
                // unwrap() is okay here.
                self.pop().unwrap();
            } else {
                break;
            }
        }
        let end = self.consume(LexemeTypes::CloseParen)?.span;
        self.consume(LexemeTypes::OpenBrace)?;
        let body = _parse(self)?;
        self.consume(LexemeTypes::CloseBrace)?;
        Ok(SyntaxNode::new(
            SyntaxNodeType::Function {
                params,
                name: nm,
                return_ty: ty,
                body,
            },
            Span::from_to(start, end),
        ))
    }

    fn parse_return(&mut self) -> Result<SyntaxNode> {
        let span = self.consume(LexemeTypes::Keyword(Keywords::Return))?.span;
        let mut expr = None;

        if let Lexeme { ty: LexemeTypes::Semicolon, .. } = self.peek()? {
        } else {
            expr = Some(self.parse_expr()?);
        }

        self.consume(LexemeTypes::Semicolon)?;
        Ok(SyntaxNode::new(SyntaxNodeType::Return { expr }, Span::from_to(span, span)))
    }
    
    fn parse_vdec(&mut self) -> Result<SyntaxNode> {
        let start = self.consume(LexemeTypes::Keyword(Keywords::Return))?.span;
        let (name, _) = self.eat_idn()?;
        self.consume(LexemeTypes::Equals)?;
        let init = self.parse_expr()?;
        let end = self.consume(LexemeTypes::Semicolon)?.span;

        Ok(SyntaxNode::new(SyntaxNodeType::VDec {
            init,
            name
        }, Span::from_to(start, end)))
    }
}

fn _parse(pb: &mut ParseBuffer) -> Result<SyntaxBlock> {
    let mut block = SyntaxBlock::new();
    loop {
        // nb. done < peek; short-circuiting
        if pb.done() || pb.peek()?.ty == LexemeTypes::CloseBrace {
            break;
        }

        let next = pb.peek()?;
        let node = match next.ty {
            LexemeTypes::Keyword(kw) => match kw {
                Keywords::Fn => pb.parse_fndef()?,
                Keywords::Return => pb.parse_return()?,
                Keywords::Let => pb.parse_vdec()?,
            },
            LexemeTypes::CloseBrace => unreachable!(),
            _ => panic!("could not parse a statement, for some reason."),
        };
        block.push(node);
    }

    Ok(block)
}

pub fn parse(lexemes: Vec<Lexeme>) -> Result<SyntaxBlock> {
    let mut parser = ParseBuffer::new(lexemes);
    let root = _parse(&mut parser)?;
    Ok(root)
}
