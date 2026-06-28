//@ # Types
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```rustylr
//@ Type(Type)
//@     : bool_! { Type::Bool }
//@     | lparen! rparen! { Type::Unit }
//@     | trait_self! { Type::TraitSelf }
//@     | amp! lifetime=Lifetime? ty=Type { Type::Ref(lifetime, Box::new(ty)) }
//@     ;
//@ ```
//@
use crate::language::*; //#
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
    TraitSelf,
    Ref(Option<Lifetime>, Box<Type>),
}
