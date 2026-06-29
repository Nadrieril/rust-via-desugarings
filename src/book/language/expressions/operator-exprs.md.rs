use crate::language::*; //#
//@ # Operator Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ OperatorExpression:
//@     | e=ArithmeticOrLogicalExpression => e,
//@     | e=AssignmentExpression => e,
//@
//@ ArithmeticOrLogicalExpression -> OperatorExpression:
//@     | a=Expression `+` b=Expression => OperatorExpression::Add(a, b),
//@
//@ AssignmentExpression -> OperatorExpression: lhs=Expression `=` rhs=Expression => OperatorExpression::Assignment(lhs, rhs)
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum OperatorExpression {
    Add(Expression, Expression),
    Assignment(Expression, Expression),
}
