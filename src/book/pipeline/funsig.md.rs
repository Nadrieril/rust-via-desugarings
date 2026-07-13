//@ # Function Signature Desugarings
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use crate::desugarings::*;

pub fn desugar_fun_sigs(program: &mut Program) -> Result<(), CompilationError> {
    program.visit_all_mut(|f: &mut Function| {
        implicit_return(f);
        shorthand_self(f);
        self_first(f)?;
        Ok(())
    })
}

//@ If the output type is not explicitly stated, it is the unit type
//@ [ref:items.fn.implicit-return].
fn implicit_return(f: &mut Function) {
    if f.return_type.is_none() {
        f.return_type = Some(Type::mk_unit())
    }
}

//@ A `self` parameter is only allowed as the first function argument [ref:items.fn.syntax].
fn self_first(f: &Function) -> Result<(), CompilationError> {
    for p in f.parameters.iter().skip(1) {
        if !matches!(p.kind, FunctionParamKind::Regular { .. }) {
            desugaring_error!("A `self` parameter is only allowed as the first function argument")
        }
    }
    Ok(())
}

//@ A `self` parameter is sugar for `self: Self`, and a `&self` parameter is sugar for `self:
//@ &Self`. [ref:associated.fn.method.self-pat-shorthands]
fn shorthand_self(f: &mut Function) {
    if let Some(p) = f.parameters.first_mut() {
        match &p.kind {
            FunctionParamKind::SelfParam {
                mutability,
                ty: None,
            } => {
                p.kind = FunctionParamKind::SelfParam {
                    mutability: *mutability,
                    ty: Some(Type::TraitSelf),
                }
            }
            FunctionParamKind::RefSelfShorthand {
                lifetime,
                mutability,
            } => {
                p.kind = FunctionParamKind::SelfParam {
                    mutability: Mutability::Immutable,
                    ty: Some(Type::Ref(
                        lifetime.clone(),
                        *mutability,
                        Box::new(Type::TraitSelf),
                    )),
                }
            }
            _ => {}
        }
    }
}
