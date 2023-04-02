use std::collections::HashMap;

use crate::errors::SemErrors;
use crate::parse::prelude::*;
use crate::prelude::*;

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

#[derive(Debug)]
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
            .ok_or(ShadowError::brief(SemErrors::CompilerNotInAScope))
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

pub fn semantic(ast: Block) -> Result<Vec<SemNode>> {
    let ast = convert_ast(ast);
    let mut buf = AnalysisBuffer::new();
    analyse(&mut buf, &ast)?;
    Ok(ast)
}
