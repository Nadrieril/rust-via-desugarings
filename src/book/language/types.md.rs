//@ # Types
//@
//@ > This section is a work-in-progress experiment about making the book executable.
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
