use sdw_lib::parse::prelude::*;

fn print_idn(string: String, idn: u64) {
    println!("{}├ {}", "│ ".repeat(idn as usize), string);
}

fn type_disp(ty: &Type) -> String {
    match ty {
        Type::Void => "void",
        Type::Int => "integer",
    }
    .to_string()
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
                    format!(
                        "a function with name '{}' which returns '{}'",
                        name,
                        type_disp(return_ty)
                    ),
                    idn,
                );
                for (idx, pm) in params.iter().enumerate() {
                    print_idn(
                        format!("- parameter {}: '{}', of type '{}'", idx, pm.0, type_disp(&pm.1)),
                        idn + 1,
                    );
                }
                print(body, idn + 1);
            }
            Node::Return { expr } => {
                print_idn("return statement".to_string(), idn);
                if let Some(expr) = expr {
                    print_idn(
                        format!("with return expression evaluating to '{}'", expr.eval()),
                        idn + 1,
                    );
                }
            }
            Node::VDec { name, init } => {
                print_idn(format!("variable '{}' declared", name), idn);
                print_idn(format!("given intialiser that evaluates to '{};", init.eval()), idn + 1);
            }
        }
    }
}
