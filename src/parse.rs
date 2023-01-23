use crate::ir::ASTNodeIR;
use crate::lex::{Keyword, Lexeme};
use std::collections::VecDeque;

macro_rules! consume {
    ( $($variant:pat),+ in $vec:expr) => {
        $(
        match $vec.pop_front() {
            Some($variant) => {},
            None => todo!("More error handling"),
            _ => todo!("Error handling code will go here :)\nThe remaining tokens were: {:?}", $vec),
        }
        )+
    };
    ( $($variant:pat),+ in $vec:expr => $then:stmt) => {
        $(
        match $vec.pop_front() {
            Some($variant) => {$then},
            None => todo!("More error handling"),
            _ => todo!("Error handling code will go here :)\nThe remaining tokens were: {:?}", $vec),
        }
        )+
    };
}

#[derive(Default, Debug)]
pub enum PrimitiveType {
    #[default]
    Void,
    Int,
}

impl PrimitiveType {
    fn from_str(from: String) -> Self {
        match from.as_str() {
            "void" => Self::Void,
            "int" => Self::Int,
            _ => todo!(
                "'Custom' variable types not implemented yet (given {})",
                from
            ),
        }
    }
}

pub trait ASTNode: std::fmt::Debug {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self
    where
        Self: Sized;
}

#[derive(Default, Debug)]
pub struct Parameter {
    pub name: String,
    pub param_type: PrimitiveType,
}

impl ASTNode for Parameter {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self {
        let mut node = Self::default();

        consume!(Lexeme::Idn(param_type) in tokens => {
            node.param_type = PrimitiveType::from_str(param_type);
        });

        consume!(Lexeme::Idn(name) in tokens => {
            node.name = name;
        });

        node
    }
}

#[derive(Default, Debug)]
pub struct Function {
    pub return_type: PrimitiveType, // TODO: expand to handle other data types
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Block,
}

impl ASTNode for Function {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self {
        let _fnkw = tokens.pop_front();
        debug_assert!(matches!(_fnkw, Some(Lexeme::Keyword(Keyword::Fn)))); // sanity check
        let mut node = Function::default();

        consume!(Lexeme::Idn(ret_type) in tokens => {
            node.return_type = PrimitiveType::from_str(ret_type)
        });

        consume!(Lexeme::Idn(name) in tokens => {
            node.name = name;
        });

        consume!(Lexeme::OpenParen in tokens);
        if !matches!(tokens.front(), Some(Lexeme::CloseParen)) {
            // fixes crash with no args; ugly
            while !tokens.is_empty() {
                node.params.push(Parameter::new(tokens));
                match tokens.front() {
                    Some(Lexeme::Delimiter) => tokens.pop_front().unwrap(),
                    _ => break,
                };
            }
        }

        consume!(Lexeme::CloseParen, Lexeme::OpenBrace in tokens);
        node.body = Block::new(tokens);
        consume!(Lexeme::CloseBrace in tokens);
        node
    }
}

#[derive(Debug, Default)]
pub struct Return {
    pub return_value: Option<u64>, // make this generic
    pub return_type: PrimitiveType,
}

impl ASTNode for Return {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self {
        let mut node = Self::default();
        consume!(Lexeme::Keyword(Keyword::Return) in tokens);
        if let Some(Lexeme::NumLiteral(inner)) = tokens.front() {
            node.return_value = Some(*inner);
            tokens.pop_front();
            node.return_type = PrimitiveType::Int;
        } else {
            node.return_type = PrimitiveType::Void;
        }
        consume!(Lexeme::Newline in tokens);
        node
    }
}

#[derive(Default, Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn ASTNodeIR>>,
}

impl ASTNode for Block {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self {
        let mut block = Self::default();

        loop {
            if let Some(tk) = tokens.front() {
                let node: Box<dyn ASTNodeIR> = match tk {
                    Lexeme::Keyword(kw) => match kw {
                        Keyword::Return => Box::new(Return::new(tokens)),
                        Keyword::Fn => Box::new(Function::new(tokens)),
                    },
                    Lexeme::NumLiteral(_) => continue,
                    Lexeme::CloseBrace => break,
                    _ => todo!(),
                };
                block.statements.push(node);
            } else {
                break;
            }
        }
        block
    }
}

// TODO: return the AST
pub fn parse(lexemes: Vec<Lexeme>) -> Block {
    let mut tokens: VecDeque<Lexeme> = VecDeque::from(lexemes);
    let root = Block::new(&mut tokens);
    root
}
