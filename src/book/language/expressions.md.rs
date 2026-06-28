//@ # Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ BlockExpression:
//@     `{` value=BooleanLiteral? `}`
//@     => value.map(BlockExpression::BoolLiteral).unwrap_or(BlockExpression::Empty)
//@
//@ BooleanLiteral:
//@     | `true` => true
//@     | `false` => false
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockExpression {
    Empty,
    BoolLiteral(bool),
}

pub type BooleanLiteral = bool;
