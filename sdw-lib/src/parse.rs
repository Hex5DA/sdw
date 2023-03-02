use crate::errors::ParseErrors;
use crate::prelude::*;

// TODO:
// - look at re-introducing NodeType enum
// - rename Node* to ASTNode*
// - parser struct / parsebuffer struct
//   - utilities similar to LexBuffer
//   - ie. eat, bump, advance, ect..
// - build up AST
// - errors

pub type LexemeStream = std::collections::VecDeque<Lexeme>;
// type Block = Vec<Node<Statement>>;

pub trait NodeTrait {
    fn new(lexemes: &mut LexemeStream) -> Result<Self>
    where
        Self: Sized;
    fn constructed_from(&self) -> LexemeStream;
    fn span(&self) -> Span;
}

#[derive(Debug)]
pub struct Node<K: NodeTrait> {
    span: Span,
    constructed_from: LexemeStream,
    ty: K,
}

impl<K: NodeTrait> Node<K> {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let inner = <K as NodeTrait>::new(lexemes)?;
        Ok(Self {
            span: inner.span(),
            constructed_from: inner.constructed_from(),
            ty: inner,
        })
    }
}

// PARSING A FUNCTION:
//
// verify first pop is Fn
// verify second pop is Idn and return
// verify third pop is Idn and return
// verify foruth pop is OpenParen
// verify fifth pop is CloseParen

// we need to be able to:
// advance the buffer
// eat / bump
// get the popped value

// -- Function::new() --
// pb: &mut ParsingBuffer
// verify!(lexemes, LexemeTypes::Keyword(Keyword::Fn))?;
// pb.verify_eat(LexemeTypes::Idn(ty), let name = ty;)?;
// pb.verify_eat(LexemeTypes::Idn(nm), let name = nm;)?;
// pb.verify(LexemeTypes::OpenParen);
// pb.verify(LexemeTypes::CloseParen);
// Node::<Block>::new(lexemes);

// notes:
// macros (??)
// ditch parsebuffer
// use macros
// need to cleanly handle generating span and constructed_from!!

// consume item from lexemes
// verify items type
// optionally run code with resultant value [?],
// report errors
// add lexeme to constructed_from
// be able to save the first and last span cleanly

#[derive(Debug)]
pub struct Function {
    name: String,
    ty: String, // TODO: PrimType enum
    span: Span,
    constructed_from: LexemeStream,
}

impl NodeTrait for Function {
    // i wrote this because i was having difficulty mapping out the necessary functions in my head
    // yes, this is horrible, yes, it will begone soon
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let mut constructed_from: Vec<Lexeme> = Vec::new();
        // the first token should always be Fn, ungraceful exit is permissible here
        let fn_kw = lexemes.pop_front().expect(
            "Function::new() has been called but the lexeme stack is empty. developer issue!",
        );
        assert!(matches!(
                fn_kw.ty,
                LexemeTypes::Keyword(Keywords::Fn)
            ),
            "Function::new() has been called but no function keyword was present on the lexeme stack. developer issue!" 
        );
        constructed_from.push(fn_kw);

        let lexeme = lexemes.pop_front().ok_or_else(|| {
            ShadowError::from_pos(
                ParseErrors::Example("example".to_string()),
                constructed_from.last().unwrap().span,
            )
        })?;

        let ty: String;
        match lexeme.ty {
            LexemeTypes::Idn(ref inner_ty) => {
                ty = inner_ty.clone();
                constructed_from.push(lexeme);
            },
            _ => {
                return Err(ShadowError::from_pos(
                    ParseErrors::Example("unexpected token".to_string()),
                    lexeme.span,
                ))
            }
        }

        let lexeme = lexemes.pop_front().ok_or_else(|| {
            ShadowError::from_pos(
                ParseErrors::Example("example".to_string()),
                constructed_from.last().unwrap().span,
            )
        })?;

        let name: String;
        match lexeme.ty {
            LexemeTypes::Idn(ref inner_ty) => {
                name = inner_ty.clone();
                constructed_from.push(lexeme);
            }
            _ => {
                return Err(ShadowError::from_pos(
                    ParseErrors::Example("unexpected token".to_string()),
                    lexeme.span,
                ))
            }
        }

        let lexeme = lexemes.pop_front().ok_or_else(|| {
            ShadowError::from_pos(
                ParseErrors::Example("example".to_string()),
                constructed_from.last().unwrap().span,
            )
        })?;
        match lexeme.ty {
            LexemeTypes::OpenParen => {},
            _ => {
                return Err(ShadowError::from_pos(
                    ParseErrors::Example("unexpected token".to_string()),
                    lexeme.span,
                ))
            }
        }
        constructed_from.push(lexeme);

        let lexeme = lexemes.pop_front().ok_or_else(|| {
            ShadowError::from_pos(
                ParseErrors::Example("example".to_string()),
                constructed_from.last().unwrap().span,
            )
        })?;
        match lexeme.ty {
            LexemeTypes::CloseParen => {},
            _ => {
                return Err(ShadowError::from_pos(
                    ParseErrors::Example("unexpected token".to_string()),
                    lexeme.span,
                ))
            }
        }
        constructed_from.push(lexeme);

        println!("function parsed, name: {:?}; type: {:?}", name, ty);
        Ok(Self {
            name,
            ty,
            span: Span::from_to(constructed_from.first().unwrap().span, constructed_from.last().unwrap().span),
            constructed_from: constructed_from.into(),
        })
    }

    fn constructed_from(&self) -> LexemeStream {
        Vec::new().into() // TODO: return constructed from, err. lifetimes
    }
    fn span(&self) -> Span {
        self.span.clone()
    }
}

pub fn parse(mut lexemes: LexemeStream) -> Result<Node<Function>> {
    Node::<Function>::new(&mut lexemes)
}
