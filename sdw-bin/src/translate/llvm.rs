use sdw_lib::consumer::prelude::*;
use sdw_lib::mangle::*;

use std::io::{Result, Write};

fn type_to_ir(ty: &Type) -> &str {
    match ty {
        Type::Void => "void",
        Type::Int => "i64",
        Type::Bool => "i1",
    }
}

fn translate_expr<W: Write>(out: &mut W, expr: &Expression) -> Result<String> {
    Ok(match &*expr.expr {
        ExpressionType::IntLit(il) => il.to_string(),
        ExpressionType::BoolLit(bl) => (*bl as u8).to_string(),
        ExpressionType::Variable(nm) => {
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
        ExpressionType::BinOp(o1, bo, o2) => {
            let temp_tag = mangle_va(format!(
                "_{}t",
                match bo {
                    BinOpTypes::Add => "a",
                    BinOpTypes::Sub => "s",
                    BinOpTypes::Div => "d",
                    BinOpTypes::Mul => "m",
                }
            ));
            let o1_tag = translate_expr(out, &o1.clone())?;
            let o2_tag = translate_expr(out, &o2.clone())?;
            writeln!(
                out,
                "  %{} = {} {} {}, {}",
                temp_tag,
                match bo {
                    BinOpTypes::Add => "add",
                    BinOpTypes::Sub => "sub",
                    BinOpTypes::Mul => "mul",
                    BinOpTypes::Div => "sdiv",
                },
                type_to_ir(&(expr).ty),
                o1_tag,
                o2_tag,
            )?;
            format!("%{}", temp_tag)
        }
        ExpressionType::Comp(o1, co, o2) => {
            let temp_tag = mangle_va("_ct".to_string());
            let o1_tag = translate_expr(out, &o1.clone())?;
            let o2_tag = translate_expr(out, &o2.clone())?;
            writeln!(
                out,
                "  %{} = icmp {} {} {}, {}",
                temp_tag,
                match co {
                    CompTypes::Equal => "eq",
                    CompTypes::NotEqual => "ne",
                    CompTypes::GreaterThan => "sgt",
                    CompTypes::GreaterThanEqualTo => "sge",
                    CompTypes::LessThan => "slt",
                    CompTypes::LessThanEqualTo => "sle",
                },
                type_to_ir(&(expr).ty),
                o1_tag,
                o2_tag,
            )?;
            format!("%{}", temp_tag)
        }
        ExpressionType::Group(gr) => translate_expr(out, gr)?,
    })
}

pub fn translate<W: Write>(out: &mut W, block: &Block) -> Result<()> {
    for node in block {
        match &node.ty {
            NodeType::Function {
                rty,
                params,
                name,
                body,
            } => {
                write!(out, "define {} @{}(", type_to_ir(&rty.inner), name.inner)?;
                let num_params = params.len();
                for (idx, param) in params.iter().enumerate() {
                    write!(out, "{} %{}", type_to_ir(&param.0.inner), param.1.inner)?;
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
            NodeType::Return { expr } => {
                if let Some(expr) = expr {
                    let tag = translate_expr(out, expr)?;
                    writeln!(out, "  ret {} {}", type_to_ir(&expr.ty), tag)?;
                } else {
                    writeln!(out, "  ret void")?;
                }
            }
            NodeType::VDec { name, init } => {
                writeln!(out, "  ; allocating '{}'", name.inner)?;
                let tag = mangle_va(name.inner.clone());
                writeln!(out, "  %{} = alloca {}", tag, type_to_ir(&init.ty),)?;
                let val_tag = translate_expr(out, init)?;
                writeln!(out, "  store {} {}, ptr %{}", type_to_ir(&init.ty), val_tag, tag,)?;
            }
        }
    }

    Ok(())
}
