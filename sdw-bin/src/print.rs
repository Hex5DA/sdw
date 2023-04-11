use sdw_lib::consumer::prelude::*;

fn print_idn(string: String, idn: u64) {
    println!("{}├ {}", "│ ".repeat(idn as usize), string);
}

fn type_disp(ty: &Type) -> String {
    match ty {
        Type::Void => "void",
        Type::Int => "integer",
        Type::Bool => "boolean",
    }
    .to_string()
}

pub fn print(block: &Block, idn: u64) {
    for node in block {
        match &node.ty {
            NodeType::Function {
                name,
                params,
                rty,
                body,
            } => {
                print_idn(
                    format!(
                        "a function with name '{}' which returns '{}'",
                        name.inner,
                        type_disp(&rty.inner)
                    ),
                    idn,
                );
                for (idx, pm) in params.iter().enumerate() {
                    print_idn(
                        format!(
                            "- parameter {}: '{}', of type '{}'",
                            idx,
                            pm.1.inner,
                            type_disp(&pm.0.inner)
                        ),
                        idn + 1,
                    );
                }
                print(body, idn + 1);
            }
            NodeType::Return { .. } => print_idn("return statement".to_string(), idn),
            NodeType::VDec { name, .. } => {
                print_idn(format!("variable '{}' declared", name.inner), idn);
                print_idn("given an intialiser".to_string(), idn + 1);
            }
            NodeType::If {
                body,
                else_block,
                else_ifs,
                ..
            } => {
                print_idn("if statement".to_string(), idn);
                print_idn("true case:".to_string(), idn + 1);
                print(body, idn + 2);

                for elif in else_ifs {
                    print_idn("else if case:".to_string(), idn + 1);
                    print(&elif.1, idn + 2)
                }

                if let Some(else_block) = else_block {
                    print_idn("else case:".to_string(), idn + 1);
                    print(else_block, idn + 2);
                }
            }
            _ => todo!("TODO: this whole module is borked; rework"),
        }
    }
}
