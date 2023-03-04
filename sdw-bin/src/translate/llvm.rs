use sdw_lib::parse::function::{Function, Parameter};
use sdw_lib::parse::prelude::*;

use std::io::{Result, Write};

use std::any::Any;

pub fn translate<T, W>(out: &mut W, node: &ASTNode<T>) -> Result<()>
where
    T: ASTNodeTrait + Clone + std::fmt::Debug + 'static,
    W: Write,
{
    // ASTNode's a generic over function data. we use std::any to somehwat-hackily match
    // over the generic values so we can use the data for translation
    //
    // this is the ideal case for a macro, but i'm not sure if one exists and i don't want
    // to have to learn how to make procedural macros just for this function :/
    let any: Box<dyn Any> = Box::new(node.ty.clone());

    if let Some(fun) = any.downcast_ref::<Function>() {
        write!(out, "define {} @{}(", fun.ty.ir_type(), fun.name)?;
        let num_params = fun.params.len();
        for (idx, param) in fun.params.iter().enumerate() {
            translate::<Parameter, W>(out, param)?;
            // slightly ugly; don't append a `,` for the last parameter
            if idx < num_params - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")")?;
    } else if let Some(param) = any.downcast_ref::<Parameter>() {
        write!(out, "{} {}", param.ty.ir_type(), param.name)?;
    } else {
        panic!("a node in the AST was not able to be translated: {:?}", node);
    }
    Ok(())
}
