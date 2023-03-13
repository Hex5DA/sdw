use crate::errors::{LexErrors, ParseErrors};
use crate::prelude::*;

pub mod prelude {
    pub use super::{Block, Expression, Node, Parameter, Spanned, Type};
}

#[derive(Debug)]
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

    pub fn ir_type(&self) -> &str {
        match self {
            Type::Void => "void",
            Type::Int => "i64",
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Void => "void",
                Type::Int => "integer",
            }
        )
    }
}

#[derive(Debug)]
pub struct Parameter(pub String, pub Type);
// TODO(5DA): stub
#[derive(Debug)]
pub struct Expression(pub i64);

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "integer literal '{}'", self.0)
    }
}

pub type Block = Vec<Box<Node>>;
#[derive(Debug)]
pub enum Node {
    Function {
        params: Vec<Parameter>,
        name: String,
        return_ty: Type,
        body: Block,
    },
    Return {
        expr: Option<Expression>,
    },
    VDec {
        name: String,
        init: Expression,
    },
}

#[derive(Debug)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
}

impl Spanned<Lexeme> {
    fn from_lexeme(lexeme: Lexeme) -> Spanned<Lexeme> {
        Spanned::<Lexeme> {
            span: lexeme.span,
            inner: lexeme,
        }
    }
}

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
#[derive(Debug)]
pub struct Parser {
    working: LexemeStream,
    lexemes: Vec<Lexeme>,
}

impl Parser {
    pub fn new(lexemes: Vec<Lexeme>) -> Self {
        assert!(!lexemes.is_empty(), "attempted to construct parser with no lexemes");
        Self {
            working: lexemes.clone().into(),
            lexemes,
        }
    }

    pub fn consume(&mut self, ty: LexemeTypes) -> Result<Spanned<Lexeme>> {
        let tk = self.pop()?;
        if tk.inner.ty == ty {
            Ok(tk)
        } else {
            Err(ShadowError::from_pos(
                ParseErrors::UnexpectedTokenEncountered(tk.inner.ty, ty),
                tk.span,
            ))
        }
    }

    fn peek(&mut self) -> Result<Spanned<Lexeme>> {
        match self.working.front() {
            Some(l) => Ok(Spanned::from_lexeme(l.clone())),
            None => Err(ShadowError::from_pos(
                ParseErrors::TokenStackEmpty,
                self.lexemes.last().unwrap().span,
            )),
        }
    }

    fn pop(&mut self) -> Result<Spanned<Lexeme>> {
        match self.working.pop_front() {
            Some(l) => Ok(Spanned::from_lexeme(l)),
            None => Err(ShadowError::from_pos(
                ParseErrors::TokenStackEmpty,
                self.lexemes.last().unwrap().span,
            )),
        }
    }

    fn eat_idn(&mut self) -> Result<(String, Spanned<Lexeme>)> {
        let tk = self.pop()?;
        match tk.inner.ty {
            LexemeTypes::Idn(ref s) => Ok((s.to_owned(), tk)),
            _ => Err(ShadowError::from_pos(
                ParseErrors::UnexpectedTokenEncountered(tk.inner.ty, LexemeTypes::Idn("<idn>".to_string())),
                tk.span,
            )),
        }
    }

    fn parse_expr(&mut self) -> Result<Expression> {
        // TODO(5DA): pratt parsing
        let tk = self.pop()?;
        match tk.inner.ty {
            LexemeTypes::Literal(Literal::Integer(i)) => Ok(Expression(i)),
            _ => Err(ShadowError::from_pos(ParseErrors::InvalidExpression, tk.span)),
        }
    }

    fn parse_fndef(&mut self) -> Result<Node> {
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
            let (ty, ty_l) = self.eat_idn()?;
            let (nm, _nm_l) = self.eat_idn()?;
            params.push(Parameter(nm, Type::from_string(ty, ty_l.span)?));
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
        self.consume(LexemeTypes::CloseParen)?;
        self.consume(LexemeTypes::OpenBrace)?;
        let body = _parse(self)?;
        self.consume(LexemeTypes::CloseBrace)?;
        Ok(Node::Function {
            params,
            name: nm,
            return_ty: Type::from_string(ty, ty_l.span)?,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Node> {
        let mut expr = None;
        // != is not supported afaik. so cope.
        if let LexemeTypes::Semicolon = self.peek()?.inner.ty {
        } else {
            expr = Some(self.parse_expr()?);
        }
        self.consume(LexemeTypes::Semicolon)?;
        Ok(Node::Return { expr })
    }

    fn parse_vdec(&mut self) -> Result<Node> {
        let (name, _) = self.eat_idn()?;
        self.consume(LexemeTypes::Equals)?;
        let init = self.parse_expr()?;
        self.consume(LexemeTypes::Semicolon)?;
        Ok(Node::VDec { name, init })
    }

    fn done(&self) -> bool {
        self.working.is_empty()
    }
}

fn _parse(parser: &mut Parser) -> Result<Block> {
    let mut block = Block::new();
    loop {
        // nb. done < peek; short-circuiting
        if parser.done() || parser.peek()?.inner.ty == LexemeTypes::CloseBrace {
            break;
        }

        let next = parser.pop()?;
        let node = match next.inner.ty {
            LexemeTypes::Keyword(kw) => match kw {
                Keywords::Fn => parser.parse_fndef()?,
                Keywords::Return => parser.parse_return()?,
                Keywords::Let => parser.parse_vdec()?,
            },
            LexemeTypes::CloseBrace => unreachable!(),
            _ => unreachable!(),
        };
        block.push(Box::new(node));
    }

    Ok(block)
}

// TODO(5DA): update all above to use Spanned<T>, only half implemented

pub fn parse(lexemes: Vec<Lexeme>) -> Result<Block> {
    let mut parser = Parser::new(lexemes);
    let root = _parse(&mut parser)?;
    Ok(root)
}
