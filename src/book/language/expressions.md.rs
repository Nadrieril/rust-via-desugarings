use crate::language::*; //#

#[path = "expressions/block-exprs.md.rs"]
pub mod block_expressions;
#[path = "expressions/call-exprs.md.rs"]
pub mod call_expressions;
#[path = "expressions/literal-exprs.md.rs"]
pub mod literal_expressions;
#[path = "expressions/operator-exprs.md.rs"]
pub mod operator_expressions;

pub use block_expressions::*;
pub use call_expressions::*;
pub use literal_expressions::*;
pub use operator_expressions::*;
//@ # Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ Expression:
//@     | expr=ExpressionWithoutBlock => expr,
//@     | expr=ExpressionWithBlock => expr,
//@
//@ ExpressionWithoutBlock -> Expression:
//@     attrs=OuterAttribute* kind=ExpressionWithoutBlockNoAttrs
//@     => Expression { attrs, kind }
//@
//@ ExpressionWithBlock -> Expression:
//@     attrs=OuterAttribute* kind=ExpressionWithBlockNoAttrs
//@     => Expression { attrs, kind }
//@
//@ ExpressionWithoutBlockNoAttrs -> ExpressionKind:
//@     | literal=LiteralExpression => ExpressionKind::Literal(literal),
//@     | path=PathExpression => ExpressionKind::Path(path),
//@     | operator=OperatorExpression => ExpressionKind::Operator(Box::new(operator)),
//@     | tuple=TupleExpression => ExpressionKind::Tuple(tuple),
//@     | call=CallExpression => ExpressionKind::Call(call),
//@
//@ ExpressionWithBlockNoAttrs -> ExpressionKind:
//@     | expr=BlockExpression => ExpressionKind::Block(expr)
//@
//@ PathExpression -> Identifier: variable=Identifier => variable
//@
//@ TupleExpression: `(` `)` => TupleExpression::Unit
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
pub struct Expression {
    pub attrs: Vec<OuterAttribute>,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum ExpressionKind {
    Literal(LiteralExpression),
    Path(PathExpression),
    Operator(Box<OperatorExpression>),
    Block(BlockExpression),
    Tuple(TupleExpression),
    Call(CallExpression),
}

pub type PathExpression = Identifier;

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum TupleExpression {
    Unit,
}
