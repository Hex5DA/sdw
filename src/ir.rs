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
    fn codegen(&self, ow: &mut OutputWrapper);
}

impl ASTNodeIR for Function {
    fn codegen(&self, ow: &mut OutputWrapper) {
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
        self.body.codegen(ow);
        ow.appendln("}".to_string(), 0);
    }
}

impl ASTNodeIR for Block {
    fn codegen(&self, ow: &mut OutputWrapper) {
        for node in &self.stmts {
            node.codegen(ow);
        }
    }
}

impl ASTNodeIR for Root {
    fn codegen(&self, ow: &mut OutputWrapper) {
        for node in &self.stmts {
            node.codegen(ow);
        }
    }
}

// return
//   - expression?
//     - some = ->.type ->.value
//     - none = void ""

impl ASTNodeIR for Statement {
    fn codegen(&self, ow: &mut OutputWrapper) {
        match &self {
            Statement::Return(inner) => {
                ow.appendln(
                    format!(
                        "ret {} {}",
                        if let Some(expr) = inner {
                            expr.evaltype()
                        } else {
                            PrimitiveType::Void
                        }.ir_type(),
                        if let Some(expr) = inner {
                            expr.eval().to_string()
                        } else {
                            "".to_string()
                        },
                    ),
                    1,
                )            
            },
            Statement::Function(func) => func.codegen(ow),
        }
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

pub fn gen_ir(ow: &mut OutputWrapper, ast: Root) {
    ast.codegen(ow);
}
