use crate::lex::{Keyword, Lexeme, Literal};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

// Functionality needed:
// - Function prototypes
// - Document Root
// - Basic statement and expression parsing
// - Integer parsing, basic typing
//
// Blocks are lists of *statements* surrounded by braces
// Statements are lines that end with a semicolon
// Expressions are items that evauluate to a value

#[derive(Default, Debug)]
pub enum PrimitiveType {
    // is this bad? this feels bad
    #[default]
    Void,
    Int,
}

impl PrimitiveType {
    fn from_str(from: String) -> Result<Self> {
        Ok(match from.as_str() {
            "void" => Self::Void,
            "int" => Self::Int,
            _ => bail!(
                "'Custom' variable types not implemented yet (given {})",
                from
            ),
        })
    }

    pub fn from_lit(from: Option<Literal>) -> Self {
        match from {
            Some(Literal::Integer(_)) => PrimitiveType::Int,
            None => PrimitiveType::Void,
        }
    }
}

macro_rules! consume {
    ( $($variant:pat),+ in $vec:expr) => {
        $(
        match $vec.pop_front() {
            Some($variant) => Ok::<(), anyhow::Error>(()),
            None => bail!("Unexpected EOF"),
            got @ _ => bail!("Expected {}, got {:?}", stringify!($variant), got),
        }
        )+
    };
    ( $variant:pat in $vec:expr => $then:stmt) => {
        match $vec.pop_front() {
            Some($variant) => Ok::<(), anyhow::Error>({$then}),
            None => bail!("Unexpected EOF"),
            got @ _ => bail!("Expected {}, got {:?}", stringify!($variant), got),
         }
    };
}

pub trait ASTNode: std::fmt::Debug {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum StatementTypes {
    Return(Expression),
    Function(Function), // If(Expression, Body)
}

impl Default for StatementTypes {
    fn default() -> Self {
        StatementTypes::Return(Expression::from_literal(Literal::Integer(0))) // TODO: this is bad, don't do this
    }
}

#[derive(Default, Debug)]
pub struct Statement {
    pub stmt_type: StatementTypes,
}

impl ASTNode for Statement {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        match lexemes.front().context("Unexpected EOF")? {
            Lexeme::Keyword(Keyword::Fn) => {
                node.stmt_type = StatementTypes::Function(Function::new(lexemes)?)
            }
            Lexeme::Keyword(Keyword::Return) => {
                consume!(Lexeme::Keyword(Keyword::Return) in lexemes)?;
                println!("k/w: {:?}", lexemes);
                let expr = Expression::new(lexemes)?;
                node.stmt_type = StatementTypes::Return(expr);
                println!("after: {:?}", lexemes);
                consume!(Lexeme::Newline in lexemes)?;
            }
            _ => todo!(),
        }
        Ok(node)
    }
}

#[derive(Debug)]
pub enum ExpressionTypes {
    Literal(Option<Literal>),
    // FunctionCall
    // Addition
    // ect
}

impl Default for ExpressionTypes {
    fn default() -> Self {
        Self::Literal(Some(Literal::Integer(0))) // TODO: this is bad, don't do this
    }
}

#[derive(Debug, Default)]
pub struct Expression {
    pub expr_type: ExpressionTypes,
}

impl Expression {
    fn from_literal(lit: Literal) -> Self {
        Self {
            expr_type: ExpressionTypes::Literal(Some(lit)),
        }
    }
}

impl ASTNode for Expression {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();
        if let Lexeme::Literal(lit) = lexemes.front().context("Unexpected EOF")? {
            node.expr_type = ExpressionTypes::Literal(Some(*lit));
            lexemes.pop_front();
        } else {
            node.expr_type = ExpressionTypes::Literal(None);
        }
        println!("expr: {:?}", lexemes);
        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Parameter {
    pub name: String,
    pub pm_type: PrimitiveType,
}

impl ASTNode for Parameter {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        consume!(Lexeme::Idn(pmt) in lexemes => {
            node.pm_type = PrimitiveType::from_str(pmt)?;
        })?;
        consume!(Lexeme::Idn(nm) in lexemes => {
            node.name = nm;
        })?;

        Ok(node)
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
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
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
                node.params.push(Parameter::new(lexemes)?);
                match lexemes.front() {
                    Some(Lexeme::Delimiter) => {
                        consume!(Lexeme::Delimiter in lexemes)?;
                    }
                    _ => break,
                }
            }
        }

        consume!(Lexeme::CloseParen in lexemes)?;
        node.body = Block::new(lexemes)?;
        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

impl ASTNode for Block {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        consume!(Lexeme::OpenBrace in lexemes)?;
        while !lexemes.is_empty() {
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
            node.stmts.push(Statement::new(lexemes)?);
        }
        consume!(Lexeme::CloseBrace in lexemes)?;

        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Root {
    pub stmts: Vec<Statement>,
}

impl ASTNode for Root {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        while !lexemes.is_empty() {
            println!("{:?}", lexemes);
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
            node.stmts.push(Statement::new(lexemes)?);
        }

        Ok(node)
    }
}

pub fn parse(lexemes: Vec<Lexeme>) -> Result<Root> {
    Root::new(&mut VecDeque::from(lexemes))
}
