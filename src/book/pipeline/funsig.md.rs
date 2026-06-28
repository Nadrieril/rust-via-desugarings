//@ # Function Signature Desugarings
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use crate::language::*;

pub fn desugar_fun_sigs(program: &mut Program) {
    for f in &mut program.functions {
        implicit_return(f);
    }
}

//@ If the output type is not explicitly stated, it is the unit
//@ type
//@ [[items.fn.implicit-return](https://doc.rust-lang.org/reference/items/functions.html#r-items.fn.implicit-return)].
fn implicit_return(f: &mut Function) {
    if f.return_type.is_none() {
        f.return_type = Some(Type::Unit)
    }
}
