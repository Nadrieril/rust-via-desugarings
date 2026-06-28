//@ # Function Signature Desugarings
use crate::language::*;

pub fn desugar_fun_sigs(program: &mut Program) {
    for f in &mut program.functions {
        desugar_fun_sig(f);
    }
}

fn desugar_fun_sig(f: &mut Function) {
    // If the output type is not explicitly stated, it is the unit type.
    //
    // https://doc.rust-lang.org/reference/items/functions.html#r-items.fn.implicit-return
    if f.return_type.is_none() {
        f.return_type = Some(Type::Unit)
    }
}
