use crate::parse::prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Parameter {
    pub name: String,
    pub ty: PrimitiveType,
    span: Span,
}

impl ASTNodeTrait for Parameter {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let mut constructed_from: Vec<Lexeme> = Vec::new();
        let ty = eat!(LexemeTypes::Idn(ref ty), ty.clone(), lexemes, constructed_from);
        let ty = PrimitiveType::from_string(ty, constructed_from.last().unwrap().span)?;
        let nm = eat!(LexemeTypes::Idn(ref nm), nm.clone(), lexemes, constructed_from);
        Ok(Self {
            name: nm,
            ty,
            span: Span::from_to(
                constructed_from.first().unwrap().span,
                constructed_from.last().unwrap().span,
            ),
        })
    }

    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Function {
    pub name: String,
    pub ty: PrimitiveType,
    pub params: Vec<ASTNode<Parameter>>,
    pub body: ASTNode<Block>,
    span: Span,
}

impl ASTNodeTrait for Function {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let mut constructed_from: Vec<Lexeme> = Vec::new();
        eat_first!(
            LexemeTypes::Keyword(Keywords::Fn),
            lexemes,
            constructed_from,
            "function"
        );
        let ty = eat!(LexemeTypes::Idn(ref ty), ty.clone(), lexemes, constructed_from);
        let ty = PrimitiveType::from_string(ty, constructed_from.last().unwrap().span)?;
        let nm = eat!(LexemeTypes::Idn(ref nm), nm.clone(), lexemes, constructed_from);
        eat!(LexemeTypes::OpenParen, lexemes, constructed_from);

        let mut params = Vec::new();
        while let Some(Lexeme {
            ty: LexemeTypes::Idn(_),
            ..
        }) = lexemes.front()
        {
            params.push(ASTNode::<Parameter>::new(lexemes)?);
            if let Some(Lexeme {
                ty: LexemeTypes::Comma, ..
            }) = lexemes.front()
            {
                lexemes.pop_front().unwrap();
            } else {
                break;
            }
        }

        eat!(LexemeTypes::CloseParen, lexemes, constructed_from);
        let body = ASTNode::<Block>::new(lexemes)?;

        Ok(Self {
            name: nm,
            params,
            ty,
            body,
            span: Span::from_to(
                constructed_from.first().unwrap().span,
                constructed_from.last().unwrap().span,
            ),
        })
    }

    fn span(&self) -> Span {
        self.span
    }
}
