use crate::language::*; //#
//@ # Literal Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ LiteralExpression:
//@     | s=STRING_LITERAL => LiteralExpression::String(s),
//@     | i=INTEGER_LITERAL => LiteralExpression::Integer(i),
//@     | `true` => LiteralExpression::Bool(true),
//@     | `false` => LiteralExpression::Bool(false),
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum LiteralExpression {
    String(String),
    Integer(u128),
    Bool(bool),
}
