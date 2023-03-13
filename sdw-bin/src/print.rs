use sdw_lib::parse::prelude::*;

fn print_idn(string: String, idn: u64) {
    println!("{}├ {}", "│ ".repeat(idn as usize), string);
}

pub fn print(block: &Block, idn: u64) {
    for node in block {
        match &**node {
            Node::Function {
                name,
                params,
                return_ty,
                body,
            } => {
                print_idn(
                    format!("a function with name '{}' which returns '{}'", name, return_ty),
                    idn,
                );
                for (idx, pm) in params.iter().enumerate() {
                    print_idn(format!("- parameter {}: '{}', of type '{}'", idx, pm.0, pm.1), idn + 1);
                }
                print(body, idn + 1);
            }
            Node::Return { expr } => {
                if let Some(expr) = expr {
                    print_idn(format!("return statement, with value {}", expr), idn);
                } else {
                    print_idn("empty return statement".to_string(), idn);
                }
            }
            Node::VDec { name, init } => {
                print_idn(format!("variable '{}' declared; initialised to {}", name, init), idn);
            }
        }
    }
}
