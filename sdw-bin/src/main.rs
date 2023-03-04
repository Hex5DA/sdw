use std::fs::{read_to_string, File};
use std::process;

use clap::Parser;

mod translate;

#[derive(Parser)]
struct Args {
    /// the file to read from
    #[arg(value_parser = port_in_range)]
    filename: String,
    /// the value to output IR to
    #[arg(default_value_t = String::from("./a.ll"))]
    out_filename: String,
}

fn port_in_range(path: &str) -> Result<String, String> {
    if path.ends_with(".sdw") {
        Ok(path.to_string())
    } else {
        Err("shadow source files should end in `.sdw`".to_string())
    }
}

fn main() {
    let args = Args::parse();
    let contents = read_to_string(&args.filename).unwrap_or_else(|err| {
        eprintln!(
            "could not read from the given file {}; error returned was\n{}",
            &args.filename, err
        );
        process::exit(1);
    });

    let lexemes = sdw_lib::lex::lex(&contents).unwrap_or_else(|err| {
        eprintln!("{}", err);
        err.verbose(&contents);
        process::exit(1);
    });

    println!("[ DBG ] lexemes recieved;\n{:#?}", lexemes);
    let ast = sdw_lib::parse::parse(lexemes.into()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        err.verbose(&contents);
        process::exit(1);
    });

    println!("[ DBG ] AST built & recieved;\n{:#?}", ast);
    let mut out = File::create(args.out_filename).unwrap_or_else(|err| {
        eprintln!("could not write to file - err: {}", err);
        process::exit(1);
    });

    translate::translate(translate::Targets::Llvm, &mut out, ast).unwrap_or_else(|err| {
        eprintln!("error whilst translating the file: {}", err);
        process::exit(1);
    });
}
