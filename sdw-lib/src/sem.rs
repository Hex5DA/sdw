use crate::errors::SemErrors;
use crate::parse::prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct SemExpression {
    expr: Expression,
    ty: Type,
}

impl SemExpression {
    fn new(expr: Expression) -> Self {
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
                expr: if let Some(expr) = expr {
                    Some(SemExpression::new(expr))
                } else {
                    None
                },
            },
            Node::VDec { name, init } => SemNode::VDec {
                name,
                init: SemExpression::new(init),
            },
        });
    }
    block
}

struct AnalysisBuffer {
    function: Option<SemNode>,
}

impl AnalysisBuffer {
    fn new() -> Self {
        Self { function: None }
    }
}

fn analyse(buf: &mut AnalysisBuffer, block: &SemBlock) -> Result<()> {
    for node in block {
        match node {
            SemNode::Function { body, .. } => {
                buf.function = Some(node.clone());
                analyse(buf, body)?;
            }
            SemNode::Return { expr, .. } => {
                if buf.function.is_none() {
                    return Err(ShadowError::brief(SemErrors::ReturnOutsideFn));
                }
                if let Some(SemNode::Function { return_ty, .. }) = &buf.function {
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
            }
            SemNode::VDec { .. } => todo!(),
        }
    }
    Ok(())
}

pub fn semantic(ast: Block) -> Result<Vec<SemNode>> {
    let ast = convert_ast(ast);
    let mut buf = AnalysisBuffer::new();
    analyse(&mut buf, &ast)?;
    Ok(ast)
}
