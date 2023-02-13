use clap::Parser;
use std::fs;
use std::process;

mod ast;
mod lex;

use ast::ir;
use ast::SymbolTable;

#[derive(Parser)]
struct Args {
    filepath: String,
    #[arg(default_value = "a.ll")]
    ofile: String,
}

fn main() {
    let args = Args::parse();
    let mut ow = ir::OutputWrapper::new(args.ofile).unwrap();
    let mut symtab = SymbolTable::new();

    let contents = fs::read_to_string(args.filepath).unwrap();
    let lexemes = lex::lex(contents).unwrap_or_else(|err| {
        eprintln!("An error occured whilst lexing the file:\n{}", err);
        process::exit(1);
    });

    println!("Lexemes recieved:\n{:#?}", lexemes);
    let ast = ast::parse(lexemes, &mut symtab).unwrap();
    println!("AST built, and recieved:\n{:#?}", ast);
    return;
    println!("Generating IR..");
    ir::gen_ir(&mut ow, &mut symtab, ast);
    ow.flush();
}
