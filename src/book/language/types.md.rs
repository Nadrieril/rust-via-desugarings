//@ # Types
//@
//@ ```lalrpop
//@ Type: Type = {
//@     "bool" => Type::Bool,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
}
