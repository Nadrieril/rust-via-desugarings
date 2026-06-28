//@ # Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```rustylr
//@ BlockExpression(BlockExpression)
//@     : lbrace! value=BooleanLiteral? rbrace! {
//@         value.map(BlockExpression::BoolLiteral).unwrap_or(BlockExpression::Empty)
//@     }
//@     ;
//@
//@ BooleanLiteral(bool)
//@     : true_! { true }
//@     | false_! { false }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockExpression {
    Empty,
    BoolLiteral(bool),
}
