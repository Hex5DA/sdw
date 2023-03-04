use sdw_lib::parse::{function::Function, ASTNode};

use std::io::{Result, Write};

mod llvm;

pub enum Targets {
    Llvm,
}

pub fn translate<W: Write>(target: Targets, out: &mut W, ast: ASTNode<Function>) -> Result<()> {
    match target {
        Targets::Llvm => llvm::translate::<Function, W>(out, &ast),
    }?;

    writeln!(out)?;
    Ok(())
}
