use clap::Parser;
use owo_colors::OwoColorize;
use sdw::lexer;
use std::fs;
use std::process;
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    /// filepath to read from
    input: String,

    /// print extra information
    #[arg(short, long)]
    verbose: bool,
}

mod print {
    use std::time::Instant;

    use owo_colors::OwoColorize;
    use sdw::prelude::*;

    macro_rules! print_idn {
        ($ident:expr, $($arg:tt)+) => {{
            let whitespace = std::iter::repeat(' ').take($ident * 2).collect::<String>();
            println!("{}{}", whitespace, format_args!($($arg)+));
        }};
    }

    fn calc(expr: &Expr) -> i64 {
        match expr {
            Expr::IntLiteral(int) => *int,
            Expr::BoolLiteral(bool) => *bool as i64,
            Expr::BiOp(op1, biop, op2) => {
                let op1 = calc(&op1.spanned);
                let op2 = calc(&op2.spanned);
                match biop {
                    BiOps::Add => op1 + op2,
                    BiOps::Sub => op1 - op2,
                    BiOps::Mul => op1 * op2,
                    BiOps::Div => op1 / op2,
                    BiOps::Mod => op1 % op2,
                    BiOps::BitOr => op1 | op2,
                    BiOps::BitAnd => op1 & op2,
                    BiOps::BitXor => op1 ^ op2,
                    BiOps::BitRshift => op1 >> op2,
                    BiOps::BitLShift => op1 << op2,
                    BiOps::LogOr => (op1 != 0 || op2 != 0) as i64,
                    BiOps::LogAnd => (op1 != 0 && op2 != 0) as i64,
                    BiOps::Eq => (op1 == op2) as i64,
                    BiOps::NEq => (op1 != op2) as i64,
                    BiOps::Gr => (op1 > op2) as i64,
                    BiOps::Ls => (op1 < op2) as i64,
                    BiOps::GrEq => (op1 >= op2) as i64,
                    BiOps::LsEq => (op1 <= op2) as i64,
                }
            }
            Expr::SubExpr(expr) => calc(&expr.spanned),
            _ => 0,
        }
    }

    fn ste(ident: usize, expr: &Expr) {
        // TO TEST: modulo
        print_idn!(ident, "[DBG] {}", calc(expr));

        match expr {
            Expr::IntLiteral(int) => print_idn!(ident, "int literal -> {}", int),
            Expr::BoolLiteral(bool) => print_idn!(ident, "boolean literal -> {}", bool),
            Expr::BiOp(op1, biop, op2) => {
                print_idn!(ident, "bi-operand expression:");
                print_idn!(
                    ident + 1,
                    "operation -> {}",
                    match biop {
                        BiOps::Add => "addition",
                        BiOps::Sub => "subtraction",
                        BiOps::Mul => "multiplication",
                        BiOps::Div => "division",
                        BiOps::Mod => "modulus",
                        BiOps::BitOr => "bitwise OR",
                        BiOps::BitAnd => "bitwise AND",
                        // BiOps::BitNot => "bitwise NOT",
                        BiOps::BitXor => "bitwise XOR",
                        BiOps::BitRshift => "bitwise right shift",
                        BiOps::BitLShift => "bitwise left shift",
                        BiOps::LogOr => "logical OR",
                        BiOps::LogAnd => "logical AND",
                        BiOps::Eq => "equal to",
                        BiOps::NEq => "not equal to",
                        BiOps::Gr => "greater than",
                        BiOps::Ls => "less than",
                        BiOps::GrEq => "greater than or equal to",
                        BiOps::LsEq => "less than or equal to",
                    }
                );

                print_idn!(ident + 1, "first operand:");
                ste(ident + 2, &op1.spanned);
                print_idn!(ident + 1, "second operand:");
                ste(ident + 2, &op2.spanned);
            },
            Expr::Cond { condition, then, elifs, r#else } => {
                print_idn!(ident, "conditional expression:");
                print_idn!(ident + 1, "condition:");
                ste(ident + 1, &condition.spanned);
                print_idn!(ident + 1, "then-case:");
                syntax_tree_ident(ident + 2, &then.spanned);
                print_idn!(ident + 1, "no. else-ifs -> {}", elifs.len());

                for elif in elifs {
                    print_idn!(ident + 1, "else-if:");
                    print_idn!(ident + 2, "condition:");
                    ste(ident + 3, &elif.0.spanned);
                    print_idn!(ident + 2, "then-case:");
                    syntax_tree_ident(ident + 3, &elif.1.spanned);
                }
            
                if let Some(r#else) = r#else {
                    print_idn!(ident + 1, "else case:");
                    syntax_tree_ident(ident + 2, &r#else.spanned);
                }
            }
            _ => todo!(),
        }
    }

    fn stb(ident: usize, bound: &Bound) {
        match bound {
            Bound::Struct(members) => {
                print_idn!(ident, "struct declaration:");
                if let Some(members) = members {
                    for member in members {
                        print_idn!(ident + 1, "member name -> {}", member.1.spanned);
                        print_idn!(ident + 1, "member type:");
                        stb(ident + 2, &member.0.spanned);
                    }
                }
            }
            Bound::Union(members) => {
                print_idn!(ident, "union declaration:");
                if let Some(members) = members {
                    for member in members {
                        print_idn!(ident + 1, "member name -> {}", member.1.spanned);
                        print_idn!(ident + 1, "member type:");
                        stb(ident + 2, &member.0.spanned);
                    }
                }
            }
            Bound::Prim(prim) => {
                print_idn!(ident, "primitive type:");
                print_idn!(
                    ident,
                    "{}",
                    match prim.spanned {
                        PrimType::Int => "int",
                        PrimType::Unt => "unt",
                        PrimType::Bool => "bool",
                        PrimType::Float => "float",
                        PrimType::String => "string",
                    }
                )
            }
            Bound::Alias(to) => {
                print_idn!(ident, "type alias:");
                print_idn!(ident, "name -> {}", to.spanned);
            }
            Bound::Pointer(to) => {
                print_idn!(ident, "pointer:");
                print_idn!(ident, "points to:");
                stb(ident + 1, &to.spanned);
            }
            Bound::FnPtr { args, return_type } => {
                print_idn!(ident, "function pointer:");
                print_idn!(ident, "return type -> {}", return_type.spanned);
                for arg in args {
                    print_idn!(ident, "arg type -> {}", arg.spanned);
                }
            }
        }
    }

    fn sts(ident: usize, stmt: &Stmt) {
        match stmt {
            Stmt::Fn {
                return_type,
                name,
                parameters,
                body,
            } => {
                print_idn!(ident, "function:");
                print_idn!(ident + 1, "name -> {}", name.spanned);
                print_idn!(ident + 1, "return type -> {}", return_type.spanned);
                print_idn!(ident + 1, "parameters:");

                for (r#type, name) in parameters {
                    print_idn!(
                        ident + 2,
                        "type, name -> {}, {}",
                        r#type.spanned,
                        name.spanned
                    );
                }
                if parameters.is_empty() {
                    print_idn!(ident + 2, "[ none ]");
                }

                print_idn!(ident + 1, "body:");
                syntax_tree_ident(ident + 2, &body.spanned);
            }
            Stmt::Stub {
                return_type,
                name,
                parameters,
            } => {
                print_idn!(ident, "function stub:");
                print_idn!(ident + 1, "name -> {}", name.spanned);
                print_idn!(ident + 1, "return type -> {}", return_type.spanned);
                print_idn!(ident + 1, "parameters:");

                for r#type in parameters {
                    print_idn!(ident + 2, "type -> {}", r#type.spanned);
                }
                if parameters.is_empty() {
                    print_idn!(ident + 2, "[ none ]");
                }
            }
            Stmt::Loop { block } => {
                print_idn!(ident, "loop:");
                syntax_tree_ident(ident + 1, block);
            }
            Stmt::Label { name } => {
                print_idn!(ident, "label:");
                print_idn!(ident + 1, "name -> {}", name.spanned);
            }
            Stmt::Goto { name } => {
                print_idn!(ident, "goto:");
                print_idn!(ident + 1, "target -> {}", name.spanned);
            }
            Stmt::Return { expr } => {
                print_idn!(ident, "return:");
                if let Some(expr) = expr {
                    ste(ident + 1, &expr.spanned);
                } else {
                    print_idn!(ident + 1, "[ no return expression ]");
                }
            }
            Stmt::VarDec { name, initialiser } => {
                print_idn!(ident, "variable declaration:");
                print_idn!(ident, "name -> {}", name.spanned);
                ste(ident + 1, &initialiser.spanned);
            }
            Stmt::VarRes { name, updated } => {
                print_idn!(ident, "variable reassignment:");
                print_idn!(ident, "name -> {}", name.spanned);
                ste(ident + 1, &updated.spanned);
            }
            Stmt::Type { name, bound } => {
                print_idn!(ident, "type declaration");
                print_idn!(ident + 1, "name -> {}", name.spanned);
                stb(ident + 1, &bound.spanned);
            }
            Stmt::Discard { .. } => todo!(),
        }
    }

    /// `ident` here is the identation for the block to be prefixed with
    /// (ie. it is the _caller's_ responsibility to `ident + 1`)
    fn syntax_tree_ident(ident: usize, root: &Block) {
        for statement in &root.stmts {
            sts(ident, &statement.spanned);
        }

        if let Some(tail) = &root.tail {
            ste(ident, &tail.spanned);
        }
    }

    pub fn syntax_tree(root: &Block) {
        syntax_tree_ident(0, root);
    }

    pub fn lexemes(lexemes: &Vec<Lexeme>) {
        println!(
            "lexemes:\n{}",
            lexemes
                .iter()
                .map(|tk| format!("{:?}", tk.spanned))
                .collect::<Vec<String>>()
                .join(" ")
        );
    }

    pub fn done(before: &Instant) {
        println!(
            "{}, in {} μs",
            "done".to_owned().bright_green(),
            before.elapsed().as_micros().bright_green(),
        );
    }
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.input).unwrap_or_else(|_| {
        eprintln!(
            "{}: could not read from input file '{}' - does it exist?",
            "error".red(),
            args.input
        );
        process::exit(1);
    });

    let mut state = sdw::common::State::new();

    let before = Instant::now();
    println!("{}..", "lexing file".bright_green());
    let lexemes = lexer::lex(&mut state, &contents);

    if !state.errors.is_empty() {
        state.print_errs(&contents, "lexing");
        process::exit(1);
    }

    print::done(&before);
    println!("produced {} lexemes", lexemes.len().bright_green());

    println!();
    if args.verbose {
        print::lexemes(&lexemes);
        println!();
    }

    let before = Instant::now();
    println!("{}..", "parsing file".bright_green());
    let st = sdw::parser::parse(&mut state, lexemes).unwrap_or_else(|err| {
        #[rustfmt::skip]
        let err_text = format!( // i don't know how better to write this. deal with it. it lines up
            r"
              an {} was raised: 
            ======================================
            ",
            "unrecoverable error".red()
        );

        eprintln!("{}", err_text);
        err.print(&contents);
        process::exit(1);
    });

    print::done(&before);
    if !state.errors.is_empty() {
        state.print_errs(&contents, "parsing");
        process::exit(1);
    }

    println!();
    if args.verbose {
        println!("ST RAW: {:?}", st);
        print::syntax_tree(&st);
        println!();
    }
}
