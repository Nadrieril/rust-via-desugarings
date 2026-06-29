use crate::language::*; //#
//@ # Call Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ CallExpression:
//@     callee=Expression `(` args=CallArgs? `)`
//@     => CallExpression { callee: Box::new(callee), args: args.unwrap_or_default() }
//@
//@ CallArgs -> Vec<Expression>:
//@     first_arg=Expression args=(`,` Expression)* `,`?
//@     => [first_arg].into_iter().chain(args).collect()
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
}
