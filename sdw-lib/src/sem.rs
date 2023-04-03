use std::collections::{HashMap, VecDeque};

use crate::errors::SemErrors;
use crate::parse::prelude::*;
use crate::prelude::*;

/*
#[derive(Debug, Clone)]
pub struct SemExpression {
    pub expr: Expression,
    pub ty: Type,
}

impl SemExpression {
    pub fn new(expr: Expression) -> Self {
        Self {
            expr,
            // TODO: type resolution
            ty: Type::Int,
        }
    }
}

pub type SemBlock = Vec<SemNode>;
#[derive(Debug, Clone)]
pub enum SemNode {
    Function {
        params: Vec<(String, Type)>,
        name: String,
        return_ty: Type,
        body: SemBlock,
    },
    Return {
        expr: Option<SemExpression>,
    },
    VDec {
        name: String,
        init: SemExpression,
    },
}

fn convert_ast(ast: Block) -> Vec<SemNode> {
    let mut block = Vec::new();
    for node in ast {
        block.push(match *node {
            Node::Function {
                params,
                name,
                return_ty,
                body,
            } => SemNode::Function {
                params,
                name,
                return_ty,
                body: convert_ast(body),
            },
            Node::Return { expr } => SemNode::Return {
                expr: expr.map(SemExpression::new),
            },
            Node::VDec { name, init } => SemNode::VDec {
                name,
                init: SemExpression::new(init),
            },
        });
    }
    block
}

#[derive(Debug)]
#[allow(dead_code)]
struct Scope {
    variables: HashMap<String, Type>,
    /// `SemNode` *must* be a `SemNode::Function`
    // (sidenote: i _pray_ enum variants as distinct types is added)
    functions: HashMap<String, SemNode>,
}

impl Scope {
    fn from_fn(fndef: SemNode) -> Self {
        assert!(matches!(fndef, SemNode::Function { .. }));
        if let SemNode::Function { ref name, .. } = fndef {
            Self {
                variables: HashMap::new(),
                functions: {
                    let mut hm = HashMap::new();
                    hm.insert(name.to_string(), fndef);
                    hm
                },
            }
        } else {
            panic!("should always be passed a SemNode::Function");
        }
    }
}

#[derive(Debug)]
struct AnalysisBuffer {
    functions: Vec<SemNode>,
    scopes: Vec<Scope>,
}

impl AnalysisBuffer {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            scopes: Vec::new(),
        }
    }

    fn analyse_return(&self, expr: &Option<SemExpression>) -> Result<()> {
        if self.functions.is_empty() {
            return Err(ShadowError::brief(SemErrors::ReturnOutsideFn));
        }
        if let SemNode::Function { return_ty, .. } = &self.functions.last().unwrap() {
            // ewwww
            match expr {
                None => {
                    if *return_ty != Type::Void {
                        return Err(ShadowError::brief(SemErrors::MismatchedFnRetTy(
                            return_ty.clone(),
                            Type::Void,
                        )));
                    }
                }
                Some(expr) => {
                    if *return_ty != expr.ty {
                        return Err(ShadowError::brief(SemErrors::MismatchedFnRetTy(
                            return_ty.clone(),
                            expr.ty.clone(),
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn scope(&mut self) -> Result<&mut Scope> {
        self.scopes
            .get_mut(0)
            .ok_or_else(|| ShadowError::brief(SemErrors::CompilerNotInAScope))
    }
}

fn analyse_expr(buf: &mut AnalysisBuffer, expr: Expression) -> Result<Type> {
    Ok(match expr {
        Expression::Variable(name) => {
            for scope in &buf.scopes {
                if let Some(ty) = scope.variables.get(&name) {
                    return Ok(ty.clone());
                }
            }
            return Err(ShadowError::brief(SemErrors::VariableNotFound(name)));
        }
        _ => Type::Int,
    })
}

fn analyse(buf: &mut AnalysisBuffer, block: &SemBlock) -> Result<()> {
    for node in block {
        match node {
            SemNode::Function { body, .. } => {
                buf.scopes.insert(0, Scope::from_fn(node.clone()));
                buf.functions.insert(0, node.clone());
                analyse(buf, body)?;
                buf.scopes.remove(0);
                buf.functions.remove(0);
            }
            SemNode::Return { expr, .. } => {
                if let Some(expr) = expr {
                    analyse_expr(buf, expr.expr.clone())?;
                }
                buf.analyse_return(expr)?
            }
            SemNode::VDec { name, init } => {
                analyse_expr(buf, init.expr.clone())?;
                buf.scope()?.variables.insert(name.to_string(), init.ty.clone());
            }
        }
    }
    Ok(())
}
*/

pub struct AnalysedExpression;

struct Scope {
    functions: HashMap<String, SyntaxNode>,
    variables: HashMap<String, Type>,
}

impl Scope {
    fn from_fn(fndef: SyntaxNode) -> Self {
        if let SyntaxNode {
            ty: SyntaxNodeType::Function { ref name, .. },
            ..
        } = fndef
        {
            Self {
                variables: HashMap::new(),
                functions: {
                    let mut hm = HashMap::new();
                    hm.insert(name.inner.clone(), fndef);
                    hm
                },
            }
        } else {
            unreachable!()
        }
    }
}

struct SemanticBuffer {
    functions: VecDeque<SyntaxNode>,
    scopes: VecDeque<Scope>,
}

impl SemanticBuffer {
    fn new() -> Self {
        Self {
            functions: VecDeque::new(),
            scopes: VecDeque::new(),
        }
    }
}

pub type AbstractBlock = Vec<AbstractNode>;

#[derive(Debug, Clone)]
pub enum AbstractNodeType {
    Function {
        name: Spanned<String>,
        params: Vec<(Spanned<Type>, Spanned<String>)>,
        rty: Spanned<Type>,
        body: AbstractBlock,
    },
    Return {
        expr: Option<Spanned<Expression>>,
    },
    VDec {
        name: Spanned<String>,
        init: Spanned<Expression>,
    },
}

impl AbstractNode {
    fn new(ty: AbstractNodeType, span: Span) -> Self {
        Self { ty, span }
    }
}

#[derive(Debug, Clone)]
pub struct AbstractNode {
    pub ty: AbstractNodeType,
    // this *might* not be necessary
    // but i'm keeping it incase i want to add a new compiler stage ^^
    span: Span,
}

fn _semantic(sb: &mut SemanticBuffer, block: SyntaxBlock) -> Result<AbstractBlock> {
    let mut analysed_block = Vec::new();
    for node in block {
        analysed_block.push(match node.ty {
            SyntaxNodeType::Function {
                ref body,
                ref rty,
                ref params,
                ref name,
            } => {
                sb.functions.push_front(node.clone());
                sb.scopes.push_front(Scope::from_fn(node.clone()));

                let body = _semantic(sb, body.to_vec())?;
                let rty = Spanned::new(rty.span, Type::from_string(rty.inner.clone(), rty.span)?);
                let mut nparams: Vec<(Spanned<Type>, Spanned<String>)> = Vec::new();
                for (t, s) in params {
                    nparams.push((
                        Spanned::new(t.span, Type::from_string(t.inner.clone(), t.span)?),
                        s.clone(),
                    ));
                }

                sb.functions.pop_front();
                sb.scopes.pop_front();

                AbstractNode::new(
                    AbstractNodeType::Function {
                        body,
                        rty,
                        params: nparams,
                        name: name.clone(),
                    },
                    node.span,
                )
            }
            SyntaxNodeType::Return { expr } => {
                // TODO: return ty validation
                if let Some(inner) = expr.inner {
                    AbstractNode::new(
                        AbstractNodeType::Return {
                            expr: Some(Spanned::new(expr.span, inner)),
                        },
                        expr.span,
                    )
                } else {
                    AbstractNode::new(AbstractNodeType::Return { expr: None }, expr.span)
                }
            }
            SyntaxNodeType::VDec { name, init } => AbstractNode::new(
                AbstractNodeType::VDec {
                    name: name.clone(),
                    init: init.clone(),
                },
                node.span,
            ),
        });
    }
    Ok(analysed_block)
}

pub fn semantic(ast: SyntaxBlock) -> Result<AbstractBlock> {
    let mut buf = SemanticBuffer::new();
    let ast = _semantic(&mut buf, ast)?;
    Ok(ast)
}
