use sdw_lib::parse::prelude::*;

use std::io::{Result, Write};

pub fn translate<W: Write>(out: &mut W, block: &Block) -> Result<()> {
    for node in block {
        // heh.. heh... . . heh. .   .     .
        match &**node {
            Node::Function {
                return_ty,
                params,
                name,
                body,
            } => {
                write!(out, "define {} @{}(", return_ty.ir_type(), name)?;
                let num_params = params.len();
                for (idx, param) in params.iter().enumerate() {
                    write!(out, "{} %{}", param.1.ir_type(), param.0)?;
                    // slightly ugly; don't append a `,` for the last parameter
                    if idx < num_params - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, ")")?;
                writeln!(out, " {{")?;
                translate::<W>(out, body)?;
                writeln!(out, "\n}}")?;
            }
            Node::Return {
                expr,
            } => {
                #[allow(clippy::write_literal)]
                // TODO(5DA): don't hardcode type
                // TODO(5DA): guarantee `expr` - semantic analysis
                if let Some(expr) = expr {
                    write!(out, "  ret {} {}", "i64", expr.0)?;
                } else {
                    write!(out, "  ret void")?;
                }
            }
        }
    }

    Ok(())
}

