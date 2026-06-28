use crate::language::*; //#
//@ # Block Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ BlockExpression:
//@     `{`
//@         inner_attrs=InnerAttribute*
//@         statements=Statement*
//@         tail=ExpressionWithoutBlock?
//@     `}`
//@     => BlockExpression { inner_attrs, statements, tail: tail.map(Box::new) }
//@
//@ BlockExpressionNoInnerAttributes -> BlockExpression:
//@     `{`
//@         statements=Statement*
//@         tail=ExpressionWithoutBlock?
//@     `}`
//@     => BlockExpression { inner_attrs: vec![], statements, tail: tail.map(Box::new) }
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
pub struct BlockExpression {
    pub inner_attrs: Vec<InnerAttribute>,
    pub statements: Vec<Statement>,
    pub tail: Option<Box<Expression>>,
}
