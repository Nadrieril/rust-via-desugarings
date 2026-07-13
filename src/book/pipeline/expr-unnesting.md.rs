//@ # Expression Unnesting
//@
//@ In this series of steps, we un-nest nested expressions by storing intermediate values in anonymous
//@ variables.
//@
//@ At the end of this series of steps, every [place
//@ context](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html) apart from
//@ simple `let`-bindings contains a side-effect-free place expression and every value context contains
//@ an operand, as defined in the [Final Language](final-language.md) section.
//@
//@ > The rest of this section is a work-in-progress experiment about making the book executable.
use crate::desugarings::*; //#

pub fn desugar_nested_exprs(program: &mut Program) -> Result<(), CompilationError> {
    explicit_value_place::make_place_coercions_explicit(program)?;
    value_to_place::desugar_value_to_place(program)
}

//@ ## Submodules
#[path = "explicit-value-place.md.rs"]
pub mod explicit_value_place;
#[path = "value-to-place.md.rs"]
pub mod value_to_place;
