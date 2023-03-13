use sdw_lib::parse::prelude::*;

use std::io::{Result, Write};

pub fn translate<W: Write>(out: &mut W, block: &Block) -> Result<()> {
    for node in block {
        // TODO(5DA): remove.
        #[allow(clippy::write_literal)]
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
                write!(out, ") ")?;
                writeln!(out, "{{")?;
                translate::<W>(out, body)?;
                writeln!(out, "}}")?;
            }
            Node::Return { expr } => {
                // TODO(5DA): don't hardcode type
                // TODO(5DA): guarantee `expr` - semantic analysis
                if let Some(expr) = expr {
                    writeln!(out, "  ret {} {}", "i64", expr.0)?;
                } else {
                    writeln!(out, "  ret void")?;
                }
            }
            Node::VDec { name, init } => {
                writeln!(out, "  %{} = alloca {}", name, "i64")?;
                writeln!(out, "  store {} {}, ptr %{}", "i64", init.0, name)?;
            }
        }
    }

    Ok(())
}
