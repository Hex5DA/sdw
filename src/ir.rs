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
        // <type> <name>
        ow.append(
            format!(
                "define {} @{} ({}) {{",
                self.return_type.ir_type(),
                self.name,
                self.params
                    .iter()
                    .map(|pm| format!("{} %{}", pm.param_type.ir_type(), pm.name))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            0,
        );
        self.body.codegen(ow);
        ow.append("}".to_string(), 0);
    }
}

impl ASTNodeIR for Block {
    fn codegen(&self, ow: &mut OutputWrapper) {
        for node in &self.statements {
            node.codegen(ow);
        }
    }
}

impl ASTNodeIR for Return {
    fn codegen(&self, ow: &mut OutputWrapper) {
        ow.append(
            format!(
                "ret {} {}",
                self.return_type.ir_type(),
                if let Some(inner) = self.return_value {
                    inner.to_string()
                } else {
                    "".to_string()
                }
            ),
            1,
        );
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
        self.file.write(vec![b' '; idnt * 2].as_slice()).unwrap();
        self.file.write(extra.as_bytes()).map(|_| ()).unwrap();
        self.file.write(&[b'\n']).unwrap();
    }

    pub fn flush(&mut self) {
        self.file.flush().map(|_| ()).unwrap();
    }
}

pub fn gen_ir(ow: &mut OutputWrapper, ast: Block) {
    ast.codegen(ow);
}
