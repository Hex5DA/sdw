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
    fn new(lexemes: &mut LexemeStream) -> Self;
    fn constructed_from(&self) -> LexemeStream;
    fn span(&self) -> Span;
}

#[derive(Debug)]
pub struct Node<K: NodeTrait> {
    span: Span,
    from: LexemeStream,
    ty: K,
}

impl<K: NodeTrait> Node<K> {
    fn new(lexemes: &mut LexemeStream) -> Self {
        let inner = <K as NodeTrait>::new(lexemes);
        Self {
            span: inner.span(),
            from: inner.constructed_from(),
            ty: inner,
        }
    }
}

#[derive(Debug)]
pub struct Function;

impl NodeTrait for Function {
    fn new(lexemes: &mut LexemeStream) -> Self {
        lexemes.pop_front(); // Fn
        let ty = lexemes.pop_front(); // <ty>
        let name = lexemes.pop_front(); // <name>
        lexemes.pop_front();
        lexemes.pop_front();
        lexemes.pop_front();
        lexemes.pop_front();
        println!("function parsed, name: {:?}; type: {:?}", name, ty);
        Self {}
    }

    fn constructed_from(&self) -> LexemeStream { todo!() }
    fn span(&self) -> Span { todo!() }
}

pub fn parse(mut lexemes: LexemeStream) -> Node<Function> {
    Node::<Function>::new(&mut lexemes)
}

