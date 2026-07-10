//@ # Desugaring Steps
//@
//@ Here is a birds-eye view of the transformations we'll be doing:
//@
//@ - Resolve names and expand macros;
//@ - Lower surface control-flow sugar (`for`, `while`, `?`, `if let`/`let else`, etc.) into a handful
//@   of constructs (`loop`, `if`, `break`/`continue`/`return`);
//@ - Make implicit operations explicit: autoderef/autoref, coercions, method resolution, operator
//@   overloading, match ergonomics;
//@ - Materialize temporaries so that every intermediate value gets a name and a lifetime.
//@ - Turn closures into plain structs;
//@ - Make drops explicit.
//@
//@ Each step must produce an equivalent program, i.e. the desugared program compiles if and only if the
//@ original one does, and both have the same semantics.
//@ Some of the desugaring steps fails to enforce that; this is noted in their Discussion section.
//@
//@ At the end of all that, we get a program in a very limited and precise subset of Rust.
//@ See [The Final Language](final-language.md) for details and discussion.
//@
//@ > The rest of this section is a work-in-progress experiment about making the book executable.
use crate::language::*;
use crate::*;

macro_rules! desugaring_error {
    ($msg:expr) => {{
        return Err(crate::CompilationError::Desugaring($msg.to_string()));
    }};
}

pub fn desugar(mut program: Program) -> Result<Program, CompilationError> {
    funsig::desugar_fun_sigs(&mut program)?;
    misc_expr_desugarings::desugar_misc_exprs(&mut program)?;
    value_to_place::desugar_value_to_place(&mut program)?;
    Ok(program)
}

//@ ## Submodules
#[path = "funsig.md.rs"]
pub mod funsig;
#[path = "minirust.md.rs"]
pub mod minirust;
#[path = "misc-expr-desugarings.md.rs"]
pub mod misc_expr_desugarings;
#[path = "value-to-place.md.rs"]
pub mod value_to_place;
