use crate::language::*; //#
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
//@     | grouped=GroupedExpression => ExpressionKind::Grouped(grouped),
//@     | tuple=TupleExpression => ExpressionKind::Tuple(tuple),
//@     | tuple_indexing=TupleIndexingExpression => ExpressionKind::TupleIndexing(tuple_indexing),
//@     | call=CallExpression => ExpressionKind::Call(call),
//@
//@ ExpressionWithBlockNoAttrs -> ExpressionKind:
//@     | expr=BlockExpression => ExpressionKind::Block(expr)
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct Expression {
    pub attrs: Vec<OuterAttribute>,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum ExpressionKind {
    Literal(LiteralExpression),
    Path(PathExpression),
    Operator(Box<OperatorExpression>),
    Grouped(Box<Expression>),
    Block(BlockExpression),
    Tuple(Vec<Expression>),
    TupleIndexing(TupleIndexingExpression),
    Call(CallExpression),
}

//@ ## Submodules
#[path = "expressions/block-exprs.md.rs"]
pub mod block_expressions;
#[path = "expressions/call-exprs.md.rs"]
pub mod call_expressions;
#[path = "expressions/grouped-exprs.md.rs"]
pub mod grouped_expressions;
#[path = "expressions/literal-exprs.md.rs"]
pub mod literal_expressions;
#[path = "expressions/operator-exprs.md.rs"]
pub mod operator_expressions;
#[path = "expressions/path-exprs.md.rs"]
pub mod path_expressions;
#[path = "expressions/tuple-exprs.md.rs"]
pub mod tuple_expressions;

pub use block_expressions::*;
pub use call_expressions::*;
pub use literal_expressions::*;
pub use operator_expressions::*;
pub use path_expressions::*;
pub use tuple_expressions::*;
