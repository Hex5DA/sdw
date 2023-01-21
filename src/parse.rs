use crate::lex::{Lexeme, Keyword};
use std::collections::VecDeque;

#[derive(Default, Debug)]
enum PrimitiveType {
    #[default]
    Void,
    Int,
}

impl PrimitiveType {
    fn from_str(from: String) -> Self {
        match from.as_str() {
            "void" => Self::Void,
            "int" => Self::Int,
            _ => todo!("'Custom' variable types not implemented yet (given {})", from),
        }
    }
}

trait ASTNode {
    fn codegen(&self);
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self;
}

#[derive(Default, Debug)]
struct Parameter {
    name: String,
    param_type: PrimitiveType,
}

impl ASTNode for Parameter {
    fn codegen(&self) {}
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self {
        let mut node = Self::default();

        if let Some(Lexeme::Idn(param_type)) = tokens.pop_front() {
            node.param_type = PrimitiveType::from_str(param_type);
        } else { todo!() }
        
        if let Some(Lexeme::Idn(name)) = tokens.pop_front() {
            node.name = name;
        } else { todo!() }

        node
    }
}

macro_rules! consume {
    ($variant:pat, $vec:expr) => {
        if let Some($variant) = $vec.pop_front() {
        } else {
            todo!("Error handling code will go here :)");
        }
    }
}

#[derive(Default, Debug)]
struct Function {
    return_type: PrimitiveType, // TODO: expand to handle other data types
    name: String,
    params: Vec<Parameter>,
}

impl ASTNode for Function {
    fn codegen(&self) {}
    fn new(tokens: &mut VecDeque<Lexeme>) -> Self {
        let _fnkw = tokens.pop_front();
        debug_assert!(matches!(_fnkw, Some(Lexeme::Keyword(Keyword::Fn)))); 
        let mut node = Function::default();

        if let Some(Lexeme::Idn(ret_type)) = tokens.pop_front() {
            node.return_type = PrimitiveType::from_str(ret_type);
        } else { todo!() }

        if let Some(Lexeme::Idn(name)) = tokens.pop_front() {
            node.name = name;
        } else { todo!() }

        consume!(Lexeme::OpenParen, tokens);        
        while let Some(tk) = tokens.front() {
            if matches!(tk, Lexeme::CloseParen) {
                break;
            }
            node.params.push(Parameter::new(tokens));
        }

        // TODO: allow consume to take multiple variants
        consume!(Lexeme::CloseParen, tokens);
        consume!(Lexeme::OpenBrace, tokens);
        while let Some(tk) = tokens.pop_front() {
            if matches!(tk, Lexeme::CloseBrace) {
                break;
            }
            // parse statements here..
        }

        node
    }
}

// TODO: return the AST
pub fn parse(lexemes: Vec<Lexeme>) {
    let mut tokens: VecDeque<Lexeme> = VecDeque::from(lexemes);
    let node = Function::new(&mut tokens); 
    println!("{:#?}", node);
}

