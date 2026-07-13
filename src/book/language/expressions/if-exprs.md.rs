use crate::language::*; //#
//@ # If Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ IfExpression:
//@     | `if` condition=Conditions then_branch=BlockExpression #[prec = `if`]
//@       => IfExpression { condition: Box::new(condition), then_branch: Box::new(then_branch.into()), else_branch: None },
//@     | `if` condition=Conditions then_branch=BlockExpression `else` else_branch=IfExpressionElse
//@       => IfExpression { condition: Box::new(condition), then_branch: Box::new(then_branch.into()), else_branch: Some(Box::new(else_branch)) },
//@
//@ Conditions -> Expression:
//@     condition=Expression => condition
//@
//@ IfExpressionElse -> Expression:
//@     | block=BlockExpression => block.into(),
//@     | if_expression=IfExpression => Expression::new(ExpressionKind::If(if_expression)),
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Option<Box<Expression>>,
}
