use sdw_lib::consumer::prelude::*;
use sdw_lib::mangle::*;

use std::io::{Result, Write};

fn type_to_ir(ty: &Type) -> &str {
    match ty {
        Type::Void => "void",
        Type::Int => "i64",
    }
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
        _ => todo!()
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
                writeln!(
                    out,
                    "  store {} {}, ptr %{}",
                    type_to_ir(&init.ty),
                    val_tag,
                    tag,
                )?;
            }
        }
    }

    Ok(())
}
