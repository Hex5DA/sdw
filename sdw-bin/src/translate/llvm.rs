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
            let tv_name = seq_mangle();
            writeln!(
                out,
                "  %{} = load {}, ptr %{}",
                tv_name,
                type_to_ir(&expr.ty),
                get_va(nm.to_string())
            )?;
            format!("%{}", tv_name)
        }
        ExpressionType::BinOp(o1, bo, o2) => {
            debug_assert_eq!(o1.ty, o2.ty);
            writeln!(
                out,
                "  ; '{}' binop",
                match bo {
                    BinOpTypes::Add => "addition",
                    BinOpTypes::Sub => "subtraction",
                    BinOpTypes::Div => "division",
                    BinOpTypes::Mul => "multiplication",
                }
            )?;

            let o1_tag = translate_expr(out, &o1.clone())?;
            let o2_tag = translate_expr(out, &o2.clone())?;
            let temp_tag = seq_mangle();
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
            debug_assert_eq!(o1.ty, o2.ty);
            let o1_tag = translate_expr(out, &o1.clone())?;
            let o2_tag = translate_expr(out, &o2.clone())?;
            let temp_tag = seq_mangle();
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
                type_to_ir(&o1.ty),
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
                // there should always be some terminating instruction,
                // so the end of the function should always be `unreachable`
                writeln!(out, "  unreachable")?;
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
                ins_va(name.inner.clone());
                let tag = get_va(name.inner.clone());
                writeln!(out, "  %{} = alloca {}", tag, type_to_ir(&init.ty),)?;
                let val_tag = translate_expr(out, init)?;
                writeln!(out, "  store {} {}, ptr %{}", type_to_ir(&init.ty), val_tag, tag,)?;
            }
            NodeType::If {
                cond,
                body,
                else_block,
                else_ifs,
            } => {
                let exit = seq_mangle();
                let cond = translate_expr(out, &cond.inner)?;

                let tt = seq_mangle();
                let mut ft = seq_mangle();
                writeln!(out, "  br i1 {}, label %{}, label %{}", cond, tt, ft)?;

                writeln!(out, "{}:", tt)?;
                translate(out, body)?;
                writeln!(out, "  br label %{}", exit)?;
                
                for elif in else_ifs {
                    let cond = translate_expr(out, &elif.0.inner)?;

                    let tt = seq_mangle();
                    writeln!(out, "{}:", ft)?;
                    ft = seq_mangle();
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cond, tt, ft)?;

                    writeln!(out, "{}:", tt)?;
                    translate(out, &elif.1)?;
                    writeln!(out, "  br label %{}", exit)?;
                }

                writeln!(out, "{}: ", ft)?;
                if let Some(eb) = else_block {
                    translate(out, eb)?;
                }

                writeln!(out, "  br label %{}", exit)?;
                writeln!(out, "{}:", exit)?;
            }
        }
    }

    Ok(())
}
