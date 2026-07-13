use crate::language::*; //#
//@ # Virtual Expressions
//@
//@ Virtual expressions are expressions that we invented for the purpose of enabling some
//@ desugarings. They don't exist in the surface language.
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum VirtualExpression {
    /// Coerce this value expression to a place expression by storing it in a temporary. See
    /// [Place-to-Value and Value-to-Place Coercions](../../pipeline/explicit-value-place.md.rs).
    ValueToPlaceCoercion(Box<Expression>),
    /// Coerce this place expression to a value expression by copying or moving out of it. See
    /// [Place-to-Value and Value-to-Place Coercions](../../pipeline/explicit-value-place.md.rs).
    PlaceToValueCoercion(Box<Expression>),
}
