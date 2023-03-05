use sdw_lib::parse::{ASTNode, Root};

use std::io::{Result, Write};

mod llvm;

pub enum Targets {
    Llvm,
}

pub fn translate<W: Write>(target: Targets, out: &mut W, ast: ASTNode<Root>) -> Result<()> {
    match target {
        Targets::Llvm => llvm::translate::<Root, W>(out, &ast),
    }?;

    writeln!(out)?;
    Ok(())
}
