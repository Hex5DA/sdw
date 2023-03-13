use std::fs::{read_to_string, File};
use std::process;

use clap::{Parser, Subcommand};

mod print;
mod translate;

#[derive(Subcommand)]
enum Commands {
    Compile {
        #[command(subcommand)]
        target: translate::Targets,
        /// the value to output IR to
        #[arg(default_value_t = String::from("./a.ll"))]
        out_filename: String,
    },
    Print,
}

#[derive(Parser)]
struct Args {
    /// the operation to run
    #[command(subcommand)]
    command: Commands,
    /// the file to read from
    #[arg(value_parser = port_in_range)]
    filename: String,
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
        err.print(&contents);
        process::exit(1);
    });

    println!(
        "[ DBG ] lexemes recieved\n{}",
        lexemes
            .iter()
            .map(|l| l.ty.to_string())
            .collect::<Vec<String>>()
            .join(" -> ")
    );

    let ast = sdw_lib::parse::parse(lexemes).unwrap_or_else(|err| {
        err.print(&contents);
        process::exit(1);
    });
    println!("[ DBG ] AST built & recieved;\n{:#?}", ast);

    match args.command {
        Commands::Compile { target, out_filename } => {
            let mut out = File::create(out_filename).unwrap_or_else(|err| {
                eprintln!("could not write to file - err: {}", err);
                process::exit(1);
            });

            translate::translate(target, &mut out, ast).unwrap_or_else(|err| {
                eprintln!("error whilst translating the file: {}", err);
                process::exit(1);
            });
        }
        Commands::Print => print::print(&ast, 0),
    }
}
