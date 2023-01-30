use crate::lex::{Keyword, Lexeme, Literal, Modifier};
use anyhow::{bail, Context, Result};
use std::collections::{HashMap, VecDeque};

pub struct Var {
    pub name: String,
    pub vtype: Option<PrimitiveType>,
    pub value: Option<Expression>,
}

pub type SymbolTable = HashMap<String, Var>;

#[derive(Default, Debug, Copy, Clone)]
pub enum PrimitiveType {
    // is this bad? this feels bad
    #[default]
    Void,
    Int,
}

impl PrimitiveType {
    pub fn from_str(from: String) -> Result<Self> {
        Ok(match from.as_str() {
            "void" => Self::Void,
            "int" => Self::Int,
            _ => bail!(
                "'Custom' variable types not implemented yet (given {})",
                from
            ),
        })
    }
}

macro_rules! consume {
    ( $variant:pat in $vec:expr => $then:stmt) => {
        match $vec.pop_front() {
            Some($variant) => Ok::<(), anyhow::Error>({$then}),
            None => bail!("Unexpected EOF"),
            got @ _ => bail!("Expected {}, got {:?}", stringify!($variant), got),
         }
    };
    ( $($variant:pat),+ in $vec:expr) => {
        $(
        consume!($variant in $vec => {})
        )+
    };
}

pub trait ASTNode: std::fmt::Debug {
    fn new(tokens: &mut VecDeque<Lexeme>, symtab: &mut SymbolTable) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum Statement {
    Return(Option<Expression>),
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
                    Some(Expression::new(lexemes, symtab)?)
                };
                consume!(Lexeme::Newline in lexemes)?;
                Self::Return(expr)
            }
            Lexeme::Keyword(Keyword::Variable) | Lexeme::Keyword(Keyword::Modifier(_)) => {
                Self::VariableDeclaration(Assignment::new(lexemes, symtab)?)
            }
            unexpected @ _ => todo!(
                "token encountered: {:?}; all tokens\n{:?}",
                unexpected,
                lexemes
            ),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String),
    Literal(Literal),
}

impl Expression {
    pub fn evaltype(&self, symtab: &mut SymbolTable) -> Result<PrimitiveType> {
        Ok(match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(_) => PrimitiveType::Int,
            },
            Self::Variable(nm) => {
                let var = symtab
                    .get(nm)
                    .context(format!("Variable {nm} not found in scope"))?;
                // var.vtype
                //    .context(format!("The variable {nm} has no strictly defined type"))?;
                if let Some(strict) = var.vtype {
                    strict
                } else {
                    var.value
                        .clone() // ew
                        .unwrap()
                        .evaltype(symtab)
                        .context("The variable's value was another variable, not yet supported")?
                }
            }
        })
    }

    pub fn eval(&self, _symtab: &mut SymbolTable) -> Result<i64> {
        Ok(match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(inner) => *inner,
            },
            Self::Variable(_) => {
                // vv constant, folding, want reference passing
                // let var = symtab.get(nm).context(format!("Variable {nm} not found in scope"))?;
                // let val = var.value.clone().context(format!("The variable {nm} has no defined value"))?;
                // val.eval(symtab)?
                unreachable!()
            }
        })
    }
}

impl ASTNode for Expression {
    fn new(lexemes: &mut VecDeque<Lexeme>, _symtab: &mut SymbolTable) -> Result<Self> {
        Ok(match lexemes.pop_front().context("Unexpected EOF")? {
            Lexeme::Literal(lit) => Self::Literal(lit),
            Lexeme::Idn(nm) => Self::Variable(nm),
            _ => bail!("Only literal expressions are supported for now!"),
        })
    }
}

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
}

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
}

pub fn parse(lexemes: Vec<Lexeme>, symtab: &mut SymbolTable) -> Result<Root> {
    Root::new(&mut VecDeque::from(lexemes), symtab)
}
