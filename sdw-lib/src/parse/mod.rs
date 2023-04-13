use crate::errors::ParseErrors;
use crate::prelude::*;

pub mod expr;
use expr::*;

pub mod prelude {
    pub use super::expr::{BinOpTypes, CompTypes, Expression};
    pub use super::{SyntaxBlock, SyntaxNode, SyntaxNodeType};
    pub use crate::common::Type;
}

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
pub type SyntaxBlock = Vec<SyntaxNode>;

#[derive(Debug, Clone)]
pub enum SyntaxNodeType {
    Function {
        name: Spanned<String>,
        params: Vec<(Spanned<String>, Spanned<String>)>,
        rty: Spanned<String>,
        body: SyntaxBlock,
    },
    Return {
        expr: Spanned<Option<Expression>>,
    },
    VDec {
        init: Spanned<Expression>,
        name: Spanned<String>,
    },
    VRes {
        new: Spanned<Expression>,
        name: Spanned<String>,
    },
    StandFnCall {
        name: Spanned<String>,
        args: Vec<Spanned<Expression>>,
    },
    If {
        cond: Spanned<Expression>,
        body: SyntaxBlock,
        else_block: Option<SyntaxBlock>,
        else_ifs: Vec<(Spanned<Expression>, SyntaxBlock)>,
    },
    Loop {
        body: SyntaxBlock,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct SyntaxNode {
    pub ty: SyntaxNodeType,
    pub span: Span,
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
        let (nm, nm_l) = self.eat_idn()?;
        self.consume(LexemeTypes::OpenParen)?;
        let mut params = Vec::new();
        while let Some(Lexeme {
            ty: LexemeTypes::Idn(_),
            ..
        }) = self.working.front()
        {
            // why does rustfmt do this :sob:
            let (ty, ty_l) = self.eat_idn()?;
            let (nm, nm_l) = self.eat_idn()?;
            params.push((Spanned::new(ty_l.span, ty), Spanned::new(nm_l.span, nm)));
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
                name: Spanned::new(nm_l.span, nm),
                rty: Spanned::new(ty_l.span, ty),
                body,
            },
            Span::from_to(start, end),
        ))
    }

    fn parse_return(&mut self) -> Result<SyntaxNode> {
        let span = self.consume(LexemeTypes::Keyword(Keywords::Return))?.span;
        let mut expr = None;

        if let Lexeme {
            ty: LexemeTypes::Semicolon,
            ..
        } = self.peek()?
        {
        } else {
            expr = Some(self.parse_expr()?);
        }

        self.consume(LexemeTypes::Semicolon)?;
        Ok(SyntaxNode::new(
            SyntaxNodeType::Return {
                expr: if let Some(expr) = expr {
                    Spanned::new(expr.span, Some(expr.inner))
                } else {
                    Spanned::new(span, None)
                },
            },
            Span::from_to(span, span),
        ))
    }

    fn parse_vdec(&mut self) -> Result<SyntaxNode> {
        let start = self.consume(LexemeTypes::Keyword(Keywords::Let))?.span;
        let (name, nm_l) = self.eat_idn()?;
        self.consume(LexemeTypes::Equals)?;
        let init = self.parse_expr()?;
        let end = self.consume(LexemeTypes::Semicolon)?.span;

        Ok(SyntaxNode::new(
            SyntaxNodeType::VDec {
                init,
                name: Spanned::new(nm_l.span, name),
            },
            Span::from_to(start, end),
        ))
    }

    fn parse_if(&mut self) -> Result<SyntaxNode> {
        let start = self.consume(LexemeTypes::Keyword(Keywords::If))?.span;
        let expr = self.parse_expr()?;
        self.consume(LexemeTypes::OpenBrace)?;
        let body = _parse(self)?;
        let mut end = self.consume(LexemeTypes::CloseBrace)?.span;

        let mut else_block = None;
        let mut else_ifs = Vec::new();
        while let Lexeme {
            ty: LexemeTypes::Keyword(Keywords::Else),
            ..
        } = self.peek()?
        {
            self.pop().unwrap();

            if let Lexeme {
                ty: LexemeTypes::Keyword(Keywords::If),
                ..
            } = self.peek()?
            {
                self.pop().unwrap();
                let expr = self.parse_expr()?;
                self.consume(LexemeTypes::OpenBrace)?;
                let block = _parse(self)?;
                end = self.consume(LexemeTypes::CloseBrace)?.span;
                else_ifs.push((expr, block));
                continue;
            }

            self.consume(LexemeTypes::OpenBrace)?;
            else_block = Some(_parse(self)?);
            end = self.consume(LexemeTypes::CloseBrace)?.span;
        }

        Ok(SyntaxNode::new(
            SyntaxNodeType::If {
                cond: expr,
                body,
                else_block,
                else_ifs,
            },
            Span::from_to(start, end),
        ))
    }

    fn parse_idn_start(&mut self) -> Result<SyntaxNode> {
        let (idn, idn_l) = self.eat_idn()?;
        let spanned = Spanned::new(idn_l.span, idn);

        Ok(
            if let Lexeme {
                ty: LexemeTypes::OpenParen,
                ..
            } = self.peek()?
            {
                self.parse_standalone_fncall(spanned)?
            } else {
                self.parse_vres(spanned)?
            },
        )
    }

    fn parse_standalone_fncall(&mut self, name: Spanned<String>) -> Result<SyntaxNode> {
        self.consume(LexemeTypes::OpenParen).unwrap();
        let mut args = Vec::new();
        if self.peek()?.ty != LexemeTypes::CloseParen {
            loop {
                args.push(self.parse_expr()?.into());
                if self.peek()?.ty == LexemeTypes::Comma {
                    self.consume(LexemeTypes::Comma)?;
                } else {
                    break;
                }
            }
        }
        self.consume(LexemeTypes::CloseParen).unwrap();
        let end = self.consume(LexemeTypes::Semicolon)?.span;
        Ok(SyntaxNode::new(
            SyntaxNodeType::StandFnCall {
                name: name.clone(),
                args,
            },
            Span::from_to(name.span, end),
        ))
    }

    fn parse_vres(&mut self, name: Spanned<String>) -> Result<SyntaxNode> {
        self.consume(LexemeTypes::Equals)?;
        let expr = self.parse_expr()?;
        let end = self.consume(LexemeTypes::Semicolon)?.span;
        Ok(SyntaxNode::new(
            SyntaxNodeType::VRes {
                new: expr,
                name: name.clone(),
            },
            Span::from_to(name.span, end),
        ))
    }

    fn parse_break(&mut self) -> Result<SyntaxNode> {
        let start = self.pop().unwrap().span;
        let end = self.consume(LexemeTypes::Semicolon)?.span;
        Ok(SyntaxNode::new(SyntaxNodeType::Break, Span::from_to(start, end)))
    }

    fn parse_continue(&mut self) -> Result<SyntaxNode> {
        let start = self.pop().unwrap().span;
        let end = self.consume(LexemeTypes::Semicolon)?.span;
        Ok(SyntaxNode::new(SyntaxNodeType::Continue, Span::from_to(start, end)))
    }

    fn parse_loop(&mut self) -> Result<SyntaxNode> {
                    let start = self.pop().unwrap().span;
            self.consume(LexemeTypes::OpenBrace)?;
            let body = _parse(self)?;
            let end = self.consume(LexemeTypes::CloseBrace)?.span;
                    Ok(SyntaxNode::new(SyntaxNodeType::Loop { body }, Span::from_to(start, end)))
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
                Keywords::If => pb.parse_if()?,
                Keywords::Else => return Err(ShadowError::from_pos(ParseErrors::ElseOutsideIf, next.span)),
                Keywords::Loop => pb.parse_loop()?,
                Keywords::Continue => pb.parse_continue()?,
                Keywords::Break => pb.parse_break()?,
            },
            LexemeTypes::Idn(_) => pb.parse_idn_start()?,
            LexemeTypes::CloseBrace => unreachable!(),
            _ => panic!("could not parse a statement, for some reason: next: {}", next.ty),
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
