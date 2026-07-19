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
//@     => BlockExpression { label: None, inner_attrs, statements, tail: tail.map(Box::new) }
//@
//@ BlockExpressionNoInnerAttributes -> BlockExpression:
//@     `{`
//@         statements=Statement*
//@         tail=ExpressionWithoutBlock?
//@     `}`
//@     => BlockExpression { label: None, inner_attrs: vec![], statements, tail: tail.map(Box::new) }
//@
//@ LabelBlockExpression -> BlockExpression:
//@     label=BlockLabel? block=BlockExpression
//@     => block.with_label(label)
//@
//@ BlockLabel -> String: label=LIFETIME `:` => label
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct BlockExpression {
    pub label: Option<String>,
    pub inner_attrs: Vec<InnerAttribute>,
    pub statements: Vec<Statement>,
    pub tail: Option<Box<Expression>>,
}

impl BlockExpression {
    pub fn with_label(mut self, label: Option<String>) -> Self {
        self.label = label;
        self
    }
}

impl From<BlockExpression> for Expression {
    fn from(block: BlockExpression) -> Self {
        Expression::new(ExpressionKind::Block(block))
    }
}
