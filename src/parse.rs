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
            _ => todo!(
                "'Custom' variable types not implemented yet (given {})",
                from
            ),
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
struct Function {
    return_type: PrimitiveType, // TODO: expand to handle other data types
    name: String,
    params: Vec<Parameter>,
}

impl ASTNode for Function {
    fn codegen(&self) {}
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
        while !tokens.is_empty() {
            node.params.push(Parameter::new(tokens));
            match tokens.front() {
                Some(Lexeme::Delimiter) => tokens.pop_front().unwrap(),
                _ => break,
            };
        }

        consume!(Lexeme::CloseParen, Lexeme::OpenBrace in tokens);
        while !tokens.is_empty() {
            // parse statements here; temporary pop
            tokens.pop_front(); 
            if let Some(Lexeme::CloseBrace) = tokens.front() {
                tokens.pop_front();
                break;
            }
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
