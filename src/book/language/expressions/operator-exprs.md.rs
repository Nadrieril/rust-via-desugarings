use crate::language::*; //#
//@ # Operator Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ OperatorExpression:
//@     | e=BorrowExpression => OperatorExpression::Borrow(e),
//@     | e=DereferenceExpression => OperatorExpression::Dereference(e),
//@     | e=ArithmeticOrLogicalExpression => e,
//@     | e=AssignmentExpression => e,
//@
//@ BorrowExpression:
//@     `&` mutability=Mutability expression=Expression #[prec = `&`]
//@     => BorrowExpression { mutability, expression: Box::new(expression) }
//@
//@ DereferenceExpression:
//@     `*` expression=Expression #[prec = `*`]
//@     => DereferenceExpression { expression: Box::new(expression) }
//@
//@ ArithmeticOrLogicalExpression -> OperatorExpression:
//@     | a=Expression `+` b=Expression => OperatorExpression::Add(a, b),
//@
//@ AssignmentExpression -> OperatorExpression: lhs=Expression `=` rhs=Expression => OperatorExpression::Assignment(lhs, rhs)
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum OperatorExpression {
    Borrow(BorrowExpression),
    Dereference(DereferenceExpression),
    Add(Expression, Expression),
    Assignment(Expression, Expression),
}

//@ The `&` (shared borrow) and `&mut` (mutable borrow) operators are unary prefix operators.
//@ [ref:expr.operator.borrow.intro].
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct BorrowExpression {
    pub mutability: Mutability,
    pub expression: Box<Expression>,
}

//@ The `*` dereference operator is applied to a pointer and denotes the pointed-to location.
//@ [ref:expr.deref.result].
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct DereferenceExpression {
    pub expression: Box<Expression>,
}
