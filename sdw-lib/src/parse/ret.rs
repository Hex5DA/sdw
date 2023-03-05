/// this file is so named to avoid conflicts with the `return` keyword
/// and no, i'm not using the raw identifier syntax. this is public-facing.
use crate::parse::prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Return {
    // optional to support empty return statements.
    expr: Option<Literal>,
    span: Span,
}

impl ASTNodeTrait for Return {
    fn new(lexemes: &mut LexemeStream) -> Result<Self> {
        let mut constructed_from = Vec::new();

        eat_first!(
            LexemeTypes::Keyword(Keywords::Return),
            lexemes,
            constructed_from,
            "return"
        );
        let mut lit = None;
        if let Some(Lexeme {
            ty: LexemeTypes::Literal(_),
            ..
        }) = lexemes.front()
        {
            lit = Some(eat!(
                LexemeTypes::Literal(ref lit),
                lit.clone(),
                lexemes,
                constructed_from
            ));
        }

        eat!(LexemeTypes::Semicolon, lexemes, constructed_from);
        Ok(Self {
            expr: lit,
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
