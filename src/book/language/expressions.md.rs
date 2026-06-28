//@ # Expressions
//@
//@ ```lalrpop
//@ BlockExpression: BlockExpression = {
//@     "{" <value:BooleanLiteral> "}" => BlockExpression::BoolLiteral(value),
//@     "{" "}" => BlockExpression::Empty,
//@ };
//@
//@ BooleanLiteral: bool = {
//@     "true" => true,
//@     "false" => false,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockExpression {
    Empty,
    BoolLiteral(bool),
}
