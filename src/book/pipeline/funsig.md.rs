//@ # Function Signature Desugarings
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use crate::language::*;

pub fn desugar_fun_sigs(program: &mut Program) {
    for f in &mut program.functions {
        implicit_return(f);
        shorthand_self(f);
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

//@ A `self` parameter is sugar for `self: Self`, and a `&self` parameter is sugar for `self:
//@ &Self`.
fn shorthand_self(f: &mut Function) {
    if let Some(p) = f.parameters.args.first_mut() {
        match &p.kind {
            FunctionParamKind::SelfParam { is_mut, ty: None } => {
                p.kind = FunctionParamKind::SelfParam {
                    is_mut: *is_mut,
                    ty: Some(Type::TraitSelf),
                }
            }
            // TODO: mutable ref
            FunctionParamKind::RefSelfShorthand {
                lifetime,
                is_mut: false,
            } => {
                p.kind = FunctionParamKind::SelfParam {
                    is_mut: false,
                    ty: Some(Type::Ref(lifetime.clone(), Box::new(Type::TraitSelf))),
                }
            }
            _ => {}
        }
    }
}
