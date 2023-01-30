use super::parse::*;
use std::fs::File;
use std::io::{BufWriter, Write};

impl PrimitiveType {
    fn ir_type(&self) -> &str {
        match self {
            Self::Int => "i64", // TODO: Support other sizes of integer
            Self::Void => "void",
        }
    }
}

pub trait ASTNodeIR: ASTNode {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable);
}

impl ASTNodeIR for Function {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        ow.appendln(
            format!(
                "define {} @{}({}) {{",
                self.return_type.ir_type(),
                self.name,
                self.params
                    .iter()
                    .map(|pm| format!("{} %{}", pm.pm_type.ir_type(), pm.name))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            0,
        );
        self.body.codegen(ow, symtab);
        ow.appendln("}".to_string(), 0);
    }
}

impl ASTNodeIR for Assignment {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        let ty = if let None = self.vtype {
            // None first because borrow checker :/
            self.value.as_ref().unwrap().evaltype(symtab).unwrap()
        } else {
            self.vtype.unwrap()
        };

        ow.appendln(format!("%{} = alloca {}", self.name, ty.ir_type()), 1);
        if let Some(val) = &self.value {
            ow.appendln(
                format!(
                    "store {} {}, ptr %{}",
                    ty.ir_type(),
                    val.eval(symtab).unwrap(),
                    self.name
                ),
                1,
            );
        }
    }
}

impl ASTNodeIR for Block {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        for node in &self.stmts {
            node.codegen(ow, symtab);
        }
    }
}

impl ASTNodeIR for Root {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        for node in &self.stmts {
            node.codegen(ow, symtab);
        }
    }
}

impl ASTNodeIR for Expression {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        match self {
            Self::Variable(nm) => {
                let var = symtab
                    .get(nm)
                    .expect(format!("Variable {nm} not found in scope").as_str());

                let intname = format!("{}deref", var.name.clone());
                ow.appendln(
                    format!(
                        "%{} = load {}, ptr %{}",
                        intname,
                        self.evaltype(symtab).unwrap().ir_type(),
                        nm.clone()
                    ),
                    1,
                );
            }
            Self::Literal(_) => {}
        }
    }
}

impl ASTNodeIR for Statement {
    fn codegen(&self, ow: &mut OutputWrapper, symtab: &mut SymbolTable) {
        let stmt = match self {
            Statement::Return(inner) => format!(
                "ret {} {}",
                if let Some(expr) = inner {
                    expr.evaltype(symtab).unwrap()
                } else {
                    PrimitiveType::Void
                }
                .ir_type(),
                match inner {
                    Some(expr) => match expr {
                        Expression::Variable(nm) => {
                            expr.codegen(ow, symtab);
                            format!("%{}deref", nm)
                        }
                        Expression::Literal(_) => expr.eval(symtab).unwrap().to_string(),
                    },
                    None => "".to_string(),
                },
            ),

            Statement::Function(func) => {
                func.codegen(ow, symtab);
                "".to_string()
            }
            Statement::VariableDeclaration(ass /* :smirk: */) => {
                ass.codegen(ow, symtab);
                "".to_string()
            }
        };
        ow.appendln(stmt, 1);
    }
}

pub struct OutputWrapper {
    file: BufWriter<File>,
}

impl OutputWrapper {
    pub fn new(path: String) -> std::io::Result<Self> {
        Ok(Self {
            file: BufWriter::new(File::create(path)?),
        })
    }

    pub fn append(&mut self, extra: String, idnt: usize) {
        self.file.write(vec![b' '; idnt * 4].as_slice()).unwrap();
        self.file.write(extra.as_bytes()).map(|_| ()).unwrap();
    }

    pub fn appendln(&mut self, extra: String, idnt: usize) {
        self.append(extra, idnt);
        self.file.write(&[b'\n']).unwrap();
    }

    pub fn flush(&mut self) {
        self.file.flush().map(|_| ()).unwrap();
    }
}

pub fn gen_ir(ow: &mut OutputWrapper, symtab: &mut SymbolTable, ast: Root) {
    ast.codegen(ow, symtab);
}
