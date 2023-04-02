use sdw_lib::consumer::prelude::*;
use sdw_lib::mangle::*;

use std::io::{Result, Write};

fn type_to_ir(ty: &Type) -> &str {
    match ty {
        Type::Void => "void",
        Type::Int => "i64",
    }
}

/// without this `translate_expr` is pretty awful
/// ignore that disgusting arg list :vomit:
macro_rules! binop_translate {
    ( $op:literal, $char:literal, $out:ident, $expr:ident, $o1:ident, $o2:ident ) => {
        {
            let temp_tag = mangle_va(format!("_{}t", $char));
            let o1_tag = translate_expr($out, &SemExpression::new(*$o1.clone()))?;
            let o2_tag = translate_expr($out, &SemExpression::new(*$o2.clone()))?;
            writeln!(
                $out,
                "  %{} = {} {} {}, {}",
                temp_tag,
                $op,
                type_to_ir(&($expr).ty),
                o1_tag,
                o2_tag,
            )?;
            format!("%{}", temp_tag)
        }
    };
}

fn translate_expr<W: Write>(out: &mut W, expr: &SemExpression) -> Result<String> {
    Ok(match &expr.expr {
        Expression::IntLit(il) => il.to_string(),
        Expression::Variable(nm) => {
            writeln!(out, "  ; dereferencing '{}'", nm)?;
            let tv_name = mangle_va(nm.to_string());
            writeln!(
                out,
                "  %{} = load {}, ptr %{}",
                tv_name,
                type_to_ir(&expr.ty),
                mangle_va_base(nm.to_string())
            )?;
            format!("%{}", tv_name)
        }
        Expression::Add(o1, o2) => binop_translate!("add", "a", out, expr, o1, o2),
        Expression::Sub(o1, o2) => binop_translate!("sub", "s", out, expr, o1, o2),
        Expression::Mul(o1, o2) => binop_translate!("mul", "m", out, expr, o1, o2),
        Expression::Div(o1, o2) => binop_translate!("sdiv", "d", out, expr, o1, o2),
        Expression::Group(gr) => translate_expr(out, &SemExpression::new(*gr.clone()))?,
    })
}

pub fn translate<W: Write>(out: &mut W, block: &Block) -> Result<()> {
    for node in block {
        match node {
            Node::Function {
                return_ty,
                params,
                name,
                body,
            } => {
                write!(out, "define {} @{}(", type_to_ir(return_ty), name)?;
                let num_params = params.len();
                for (idx, param) in params.iter().enumerate() {
                    write!(out, "{} %{}", type_to_ir(&param.1), param.0)?;
                    // slightly ugly; don't append a `,` for the last parameter
                    if idx < num_params - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, ") ")?;
                writeln!(out, "{{")?;
                translate::<W>(out, body)?;
                writeln!(out, "}}")?;
            }
            Node::Return { expr } => {
                if let Some(expr) = expr {
                    let tag = translate_expr(out, &expr)?;
                    writeln!(out, "  ret {} {}", type_to_ir(&expr.ty), tag)?;
                } else {
                    writeln!(out, "  ret void")?;
                }
            }
            Node::VDec { name, init } => {
                writeln!(out, "  ; allocating '{}'", name)?;
                let tag = mangle_va(name.to_string());
                writeln!(out, "  %{} = alloca {}", tag, type_to_ir(&init.ty),)?;
                let val_tag = translate_expr(out, &init)?;
                writeln!(out, "  store {} {}, ptr %{}", type_to_ir(&init.ty), val_tag, tag,)?;
            }
        }
    }

    Ok(())
}
