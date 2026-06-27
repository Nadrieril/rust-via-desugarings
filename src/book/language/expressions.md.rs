//@ # Expressions
//@
//@ ```lalrpop
//@ BlockExpression: BlockExpression = {
//@     "{" <value:BooleanLiteral> "}" => BlockExpression::BoolLiteral(value),
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
    BoolLiteral(bool),
}
