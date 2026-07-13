use crate::language::*; //#
//@ # Tuple and Tuple Indexing Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ TupleExpression -> Vec<Expression>:
//@     `(` elements=TupleElements? `)`
//@     => elements.unwrap_or_default()
//@
//@ TupleElements -> Vec<Expression>:
//@     elements=(Expression `,`)+ last=Expression?
//@     => elements.into_iter().chain(last).collect()
//@
//@ TupleIndexingExpression:
//@     expression=Expression `.` index=TupleIndex #[prec = `.`]
//@     => TupleIndexingExpression { expression: Box::new(expression), index }
//@
//@ TupleIndex -> usize:
//@     index=INTEGER_LITERAL => usize::try_from(index).unwrap()
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct TupleIndexingExpression {
    pub expression: Box<Expression>,
    pub index: usize,
}
