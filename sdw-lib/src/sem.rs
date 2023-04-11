use std::collections::{HashMap, VecDeque};

use crate::errors::SemErrors;
use crate::parse::prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum AbstractExpressionType {
    IntLit(i64),
    BoolLit(bool),
    Variable(String),
    FnCall(String, Vec<AbstractExpression>),
    BinOp(AbstractExpression, BinOpTypes, AbstractExpression),
    Comp(AbstractExpression, CompTypes, AbstractExpression),
    Group(AbstractExpression),
}

#[derive(Debug, Clone)]
pub struct AbstractExpression {
    pub expr: Box<AbstractExpressionType>,
    pub span: Span,
    pub ty: Type,
}

impl AbstractExpression {
    fn new(expr: AbstractExpressionType, span: Span, ty: Type) -> Self {
        Self {
            expr: Box::new(expr),
            span,
            ty,
        }
    }
}

#[allow(dead_code)] // TODO: remove
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

    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: HashMap::new(),
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

    fn scope(&mut self, span: Span) -> Result<&mut Scope> {
        self.scopes
            .get_mut(0)
            .ok_or_else(|| ShadowError::from_pos(SemErrors::CompilerNotInAScope, span))
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
        expr: Option<AbstractExpression>,
    },
    VDec {
        name: Spanned<String>,
        init: AbstractExpression,
    },
    If {
        cond: Spanned<AbstractExpression>,
        body: AbstractBlock,
        else_block: Option<AbstractBlock>,
        else_ifs: Vec<(Spanned<AbstractExpression>, AbstractBlock)>,
    },
    VRes {
        new: AbstractExpression,
        name: Spanned<String>,
    },
    StandFnCall {
        name: Spanned<String>,
        args: Vec<AbstractExpression>,
        rty: Type,
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
    pub span: Span,
}

fn expression(sb: &mut SemanticBuffer, expr: Expression, span: Span) -> Result<AbstractExpression> {
    Ok(match expr {
        Expression::BoolLit(bl) => AbstractExpression::new(AbstractExpressionType::BoolLit(bl), span, Type::Bool),
        Expression::IntLit(il) => AbstractExpression::new(AbstractExpressionType::IntLit(il), span, Type::Int),
        Expression::BinOp(o1, bo, o2) => {
            let o1 = expression(sb, *o1.inner, o1.span)?;
            let o2 = expression(sb, *o2.inner, o2.span)?;

            let span = Span::from_to(o1.span, o2.span);
            let ty = o1.ty.clone();
            if o1.ty != o2.ty {
                return Err(ShadowError::from_pos(SemErrors::MismatchedTypes(o1.ty, o2.ty), span));
            }

            AbstractExpression::new(AbstractExpressionType::BinOp(o1, bo, o2), span, ty)
        }
        Expression::Comp(o1, co, o2) => {
            let o1 = expression(sb, *o1.inner, o1.span)?;
            let o2 = expression(sb, *o2.inner, o2.span)?;

            let span = Span::from_to(o1.span, o2.span);
            if o1.ty != o2.ty {
                return Err(ShadowError::from_pos(SemErrors::MismatchedTypes(o1.ty, o2.ty), span));
            }

            AbstractExpression::new(AbstractExpressionType::Comp(o1, co, o2), span, Type::Bool)
        }
        Expression::Group(gp) => {
            let expr = expression(sb, *gp.inner, gp.span)?;
            let ty = expr.ty.clone();
            AbstractExpression::new(AbstractExpressionType::Group(expr), gp.span, ty)
        }
        Expression::Variable(name) => {
            for scope in &sb.scopes {
                if let Some(ty) = scope.variables.get(&name) {
                    return Ok(AbstractExpression::new(
                        AbstractExpressionType::Variable(name),
                        span,
                        ty.clone(),
                    ));
                }
            }
            if let Some(SyntaxNode {
                ty: SyntaxNodeType::Function { params, .. },
                ..
            }) = sb.functions.get(0)
            {
                for param in params {
                    if name == param.1.inner {
                        return Ok(AbstractExpression::new(
                            AbstractExpressionType::Variable(name),
                            span,
                            Type::from_string(param.0.inner.clone(), param.0.span)?,
                        ));
                    }
                }
            }

            return Err(ShadowError::from_pos(SemErrors::VariableNotFound(name), span));
        }
        Expression::FnCall(name, args) => {
            let fname = name.clone(); // prevent shadowing
            let mut nargs = Vec::new();
            for a in &args {
                nargs.push(expression(sb, *a.inner.clone(), a.span)?);
            }
            for fndef in &sb.functions {
                if let SyntaxNode {
                    ty: SyntaxNodeType::Function { params, rty, name, .. },
                    ..
                } = fndef
                {
                    if name.inner == fname {
                        if nargs.len() != params.len() {
                            return Err(ShadowError::from_pos(
                                SemErrors::MismatchedNumArgs(params.len(), args.len()),
                                span,
                            ));
                        }

                        for (arg, param) in nargs.iter().zip(params) {
                            let pty = Type::from_string(param.0.inner.clone(), param.0.span)?;
                            if arg.ty != pty {
                                return Err(ShadowError::from_pos(
                                    SemErrors::ArgTyMismatch(pty, arg.ty.clone()),
                                    arg.span,
                                ));
                            }
                        }
                        return Ok(AbstractExpression::new(
                            AbstractExpressionType::FnCall(fname, nargs),
                            span,
                            Type::from_string(rty.inner.clone(), rty.span)?,
                        ));
                    }
                } else {
                    unreachable!()
                }
            }
            return Err(ShadowError::from_pos(SemErrors::FunctionNotFound(name), span));
        }
    })
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
                sb.scopes.push_front(Scope::from_fn(node.clone()));
                sb.functions.push_front(node.clone());

                let body = _semantic(sb, body.to_vec())?;
                let rty = Spanned::new(rty.span, Type::from_string(rty.inner.clone(), rty.span)?);
                let mut nparams: Vec<(Spanned<Type>, Spanned<String>)> = Vec::new();
                for (t, s) in params {
                    nparams.push((
                        Spanned::new(t.span, Type::from_string(t.inner.clone(), t.span)?),
                        s.clone(),
                    ));
                }

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
                // pretend you can't see the RTYV here :vomit:
                let rty = match sb.functions.iter().last() {
                    Some(SyntaxNode {
                        ty: SyntaxNodeType::Function { rty, .. },
                        ..
                    }) => Type::from_string(rty.inner.clone(), rty.span)?,
                    Some(_) => unreachable!(),
                    None => return Err(ShadowError::from_pos(SemErrors::ReturnOutsideFn, node.span)),
                };

                if let Some(inner) = expr.inner {
                    let inner = expression(sb, inner, expr.span)?;
                    if rty != inner.ty {
                        return Err(ShadowError::from_pos(
                            SemErrors::MismatchedFnRetTy(rty, inner.ty),
                            expr.span,
                        ));
                    }

                    AbstractNode::new(AbstractNodeType::Return { expr: Some(inner) }, expr.span)
                } else {
                    if rty != Type::Void {
                        return Err(ShadowError::from_pos(
                            SemErrors::MismatchedFnRetTy(rty, Type::Void),
                            expr.span,
                        ));
                    }

                    AbstractNode::new(AbstractNodeType::Return { expr: None }, expr.span)
                }
            }
            SyntaxNodeType::VDec { name, init } => {
                let expr = expression(sb, init.inner, init.span)?;
                sb.scope(init.span)?
                    .variables
                    .insert(name.inner.clone(), expr.ty.clone());

                AbstractNode::new(
                    AbstractNodeType::VDec {
                        name: name.clone(),
                        init: expr,
                    },
                    node.span,
                )
            }
            SyntaxNodeType::If {
                cond,
                body,
                else_block,
                else_ifs,
            } => {
                let cond = Spanned::new(cond.span, expression(sb, cond.inner, cond.span)?);
                if cond.inner.ty != Type::Bool {
                    return Err(ShadowError::from_pos(SemErrors::CondNotBool(cond.inner.ty), cond.span));
                }

                sb.scopes.push_front(Scope::new());
                let body = _semantic(sb, body)?;
                sb.scopes.pop_front();

                let else_block = if let Some(else_block) = else_block {
                    sb.scopes.push_front(Scope::new());
                    let binding = _semantic(sb, else_block)?;
                    sb.scopes.pop_front();
                    Some(binding)
                } else {
                    None
                };

                let mut nelse_ifs = Vec::new();
                for elif in else_ifs {
                    nelse_ifs.push((
                        Spanned::new(elif.0.span, expression(sb, elif.0.inner, elif.0.span)?),
                        _semantic(sb, elif.1)?,
                    ))
                }

                AbstractNode::new(
                    AbstractNodeType::If {
                        cond,
                        body,
                        else_block,
                        else_ifs: nelse_ifs,
                    },
                    node.span,
                )
            }
            SyntaxNodeType::VRes { new, name } => {
                let new = expression(sb, new.inner, new.span)?;
                let mut rval = None;
                for scope in &sb.scopes {
                    if let Some(ty) = scope.variables.get(&name.inner) {
                        if ty != &new.ty {
                            return Err(ShadowError::from_pos(
                                SemErrors::CannotReassignVariableType(name.inner, ty.clone(), new.ty),
                                node.span,
                            ));
                        } else {
                            rval = Some(AbstractNode::new(
                                AbstractNodeType::VRes {
                                    new,
                                    name: name.clone(),
                                },
                                node.span,
                            ));
                            break;
                        }
                    }
                }
                if let Some(rval) = rval {
                    rval
                } else {
                    return Err(ShadowError::from_pos(
                        SemErrors::VariableNotFound(name.inner),
                        node.span,
                    ));
                }
            }
            SyntaxNodeType::StandFnCall { name, args } => {
                // TODO: this is staright copied from expression()
                let fname = name.clone(); // prevent shadowing
                let mut nargs = Vec::new();
                let mut rval = None;
                for a in &args {
                    nargs.push(expression(sb, a.inner.clone(), a.span)?);
                }
                for fndef in &sb.functions {
                    if let SyntaxNode {
                        ty: SyntaxNodeType::Function { params, rty, name, .. },
                        ..
                    } = fndef
                    {
                        if name.inner == fname.inner {
                            if nargs.len() != params.len() {
                                return Err(ShadowError::from_pos(
                                    SemErrors::MismatchedNumArgs(params.len(), args.len()),
                                    node.span,
                                ));
                            }

                            for (arg, param) in nargs.iter().zip(params) {
                                let pty = Type::from_string(param.0.inner.clone(), param.0.span)?;
                                if arg.ty != pty {
                                    return Err(ShadowError::from_pos(
                                        SemErrors::ArgTyMismatch(pty, arg.ty.clone()),
                                        arg.span,
                                    ));
                                }
                            }
                            rval = Some(AbstractNode::new(
                                AbstractNodeType::StandFnCall {
                                    name: name.clone(),
                                    args: nargs.clone(),
                                    rty: Type::from_string(rty.inner.clone(), rty.span)?,
                                },
                                node.span,
                            ));
                        }
                    } else {
                        unreachable!()
                    }
                }
                if let Some(rval) = rval {
                    rval
                } else {
                    return Err(ShadowError::from_pos(
                        SemErrors::FunctionNotFound(name.inner),
                        node.span,
                    ));
                }
            }
        });
    }

    Ok(analysed_block)
}

pub fn semantic(ast: SyntaxBlock) -> Result<AbstractBlock> {
    let mut buf = SemanticBuffer::new();
    let ast = _semantic(&mut buf, ast)?;
    Ok(ast)
}
