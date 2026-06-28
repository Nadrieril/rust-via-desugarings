//@ # Types
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ Type:
//@     | `bool` => Type::Bool
//@     | `(` `)` => Type::Unit
//@     | `Self` => Type::TraitSelf
//@     | `&` lifetime=Lifetime? m=Mutability ty=Type => Type::Ref(lifetime, m, Box::new(ty))
//@ ```
//@
use crate::language::*; //#
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
    TraitSelf,
    Ref(Option<Lifetime>, Mutability, Box<Type>),
}
