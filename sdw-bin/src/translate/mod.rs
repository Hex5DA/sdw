use sdw_lib::consumer::Block;

use std::io::{Result, Write};

use clap::Subcommand;

mod llvm;

#[derive(Clone, Subcommand)]
pub enum Targets {
    Llvm,
}

pub fn translate<W: Write>(target: Targets, out: &mut W, ast: Block) -> Result<()> {
    match target {
        Targets::Llvm => llvm::translate::<W>(out, &ast),
    }?;

    writeln!(out)?;
    Ok(())
}
