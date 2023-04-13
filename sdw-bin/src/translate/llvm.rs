use sdw_lib::consumer::prelude::*;
use sdw_lib::mangle::*;

use std::io::{Result, Write};
use std::sync::Mutex;

/// (start_tag, end_tag)
static LOOPS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

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
        ExpressionType::FnCall(nm, args) => {
            let mut args_string = String::new();
            for (idx, arg) in args.iter().enumerate() {
                let tag = translate_expr(out, arg)?;
                args_string.push_str(format!("{} {}", type_to_ir(&arg.ty), tag).as_str());

                if idx < args.len() - 1 {
                    args_string.push_str(", ");
                }
            }

            let rvt = seq_mangle();
            writeln!(
                out,
                "  %{} = call {} @{}({})",
                rvt,
                type_to_ir(&expr.ty),
                nm,
                args_string
            )?;
            format!("%{}", rvt)
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
                for param in params {
                    ins_va(param.1.inner.clone());
                    let tag = get_va(param.1.inner.clone());
                    writeln!(out, "  %{} = alloca {}", tag, type_to_ir(&param.0.inner))?;
                    writeln!(
                        out,
                        "  store {} %{}, ptr %{}",
                        type_to_ir(&param.0.inner),
                        param.1.inner,
                        tag
                    )?;
                }
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
                writeln!(out, "  ; conditional (if) statement begin")?;
                // this is poor, because we the tag for exit will increase
                // compared to the other tags. i don't think this is a avoidable without some
                // form of lazy-loading , since we need it immediately :/
                let exit = seq_mangle();
                let cond = translate_expr(out, &cond.inner)?;

                // true tag
                let tt = seq_mangle();
                // false tag
                let mut ft = seq_mangle();
                writeln!(out, "  br i1 {}, label %{}, label %{}", cond, tt, ft)?;

                writeln!(out, "; true case")?;
                writeln!(out, "{}:", tt)?;
                translate(out, body)?;
                writeln!(out, "  br label %{}", exit)?;

                for elif in else_ifs {
                    writeln!(out, "; false case")?;
                    // we use the previous false tag, giving a chaining effect
                    writeln!(out, "{}:", ft)?;

                    let cond = translate_expr(out, &elif.0.inner)?;
                    // we update it here for the next iteration (next `else if`) if there is one
                    ft = seq_mangle();
                    let tt = seq_mangle();
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cond, tt, ft)?;

                    writeln!(out, "{}:", tt)?;
                    translate(out, &elif.1)?;
                    writeln!(out, "  br label %{}", exit)?;
                }

                // write the final tag. if there is no `else`, this tag is wasted, but it removes
                // some nasty code in the above loop so i'm okay with it.
                // this code would need to check if it was the last iteration of the loop, and if
                // it was use exit as the false-case. as it is, there's always a dangling tag, so
                // this cleans it up.
                writeln!(out, "; false case")?;
                writeln!(out, "{}: ", ft)?;
                if let Some(eb) = else_block {
                    // if there is an else block, the above oversight actually becomes useful, as
                    // the final elif will attempt to branch to a dangling tag. we can just write
                    // the else block beneath it, to get the expected behaviour.
                    translate(out, eb)?;
                }

                // in both cases, we can use a final branch to get to the end of the conditional.
                writeln!(out, "  br label %{}", exit)?;
                writeln!(out, "; conditional exit label")?;
                writeln!(out, "{}:", exit)?;
            }
            NodeType::VRes { new, name } => {
                let tag = translate_expr(out, new)?;
                let ntag = get_va(name.inner.clone());
                writeln!(out, "  store {} {}, ptr %{}", type_to_ir(&new.ty), tag, ntag)?;
            }
            NodeType::StandFnCall { name, args, rty } => {
                let mut args_string = String::new();
                for (idx, arg) in args.iter().enumerate() {
                    let tag = translate_expr(out, arg)?;
                    args_string.push_str(format!("{} {}", type_to_ir(&arg.ty), tag).as_str());

                    if idx < args.len() - 1 {
                        args_string.push_str(", ");
                    }
                }

                writeln!(out, "  call {} @{}({})", type_to_ir(&rty), name.inner, args_string)?;
            }
            NodeType::Loop { body } => {
                let lp_start = seq_mangle();
                let lp_end = seq_mangle();
                LOOPS.lock().unwrap().push((lp_start.clone(), lp_end.clone()));
                writeln!(out, "  br label %{}", lp_start)?;
                writeln!(out, "; loop")?;
                writeln!(out, "{}:", lp_start)?;
                translate(out, body)?;
                writeln!(out, "  br label %{}", lp_start)?;
                writeln!(out, "{}:", lp_end)?;
            }
            NodeType::Continue => {
                let lp_start = LOOPS.lock().unwrap().last().unwrap().0.clone();
                writeln!(out, "  br label %{}", lp_start)?;
            }
            NodeType::Break => {
                let lp_end = &LOOPS.lock().unwrap().last().unwrap().1.clone();
                writeln!(out, "  br label %{}", lp_end)?;
            }
        }
        // notes on how to translate looping:
        // <https://godbolt.org/z/6zjEoMEhn>
    }

    Ok(())
}
