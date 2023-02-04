use crate::lex::Lexeme;
use anyhow::{bail, Result};
use std::collections::{HashMap, VecDeque};

mod expression;
mod function;
mod statement;
mod variables;

use expression::Expression;
use ir::OutputWrapper;
use statement::Root;

pub struct Var {
    pub name: String,
    pub vtype: Option<PrimitiveType>,
    pub value: Option<Box<dyn Expression>>,
}

pub type SymbolTable = HashMap<String, Var>;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
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

    fn ir_type(&self) -> &str {
        match self {
            Self::Int => "i64", // TODO: Support other sizes of integer
            Self::Void => "void",
        }
    }
}

#[macro_export]
macro_rules! consume {
    ( $variant:pat in $vec:expr => $then:stmt) => {
        match $vec.pop_front() {
            Some($variant) => Ok::<(), anyhow::Error>({$then}),
            None => bail!("Unexpected EOF"),
            got => bail!("Expected {}, got {:?}. Remaining tokens were:\n{:?}", stringify!($variant), got, $vec),
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

    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable);
}

pub fn parse(lexemes: Vec<Lexeme>, symtab: &mut SymbolTable) -> Result<Root> {
    Root::new(&mut VecDeque::from(lexemes), symtab)
}

pub mod ir {
    use super::{ASTNode, Root, SymbolTable};
    use std::{
        fs::File,
        io::{BufWriter, Write},
    };
    pub struct OutputWrapper {
        file: BufWriter<File>,
    }

    impl OutputWrapper {
        pub fn new(path: String) -> std::io::Result<Self> {
            Ok(Self {
                file: BufWriter::new(File::create(path)?),
            })
        }

        pub fn append(&mut self, extra: String, idnt: usize) {
            self.file
                .write_all(vec![b' '; idnt * 4].as_slice())
                .unwrap();
            self.file.write(extra.as_bytes()).map(|_| ()).unwrap();
        }

        pub fn appendln(&mut self, extra: String, idnt: usize) {
            self.append(extra, idnt);
            self.file.write_all(&[b'\n']).unwrap();
        }

        pub fn flush(&mut self) {
            self.file.flush().map(|_| ()).unwrap();
        }
    }

    pub fn gen_ir(ow: &mut OutputWrapper, symtab: &mut SymbolTable, ast: Root) {
        ast.codegen(ow, symtab)
    }
}
