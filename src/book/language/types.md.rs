//@ # Types
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```rustylr
//@ Type(Type)
//@     : bool_! {
//@         Type::Bool
//@     }
//@     | lparen! rparen! {
//@         Type::Unit
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
}
