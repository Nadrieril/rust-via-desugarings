use crate::language::*; //#
//@ # Path Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ PathExpression: segment=Identifier
//@     => PathExpression::SingleSegment(segment)
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum PathExpression {
    SingleSegment(Identifier),
}
