//@ # Types
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ Type:
//@     | `bool` => Type::Bool
//@     | `str` => Type::Str
//@     | `(` `)` => Type::Unit
//@     | `Self` => Type::TraitSelf
//@     | `&` lifetime=Lifetime? m=Mutability ty=Type => Type::Ref(lifetime, m, Box::new(ty))
//@ ```
//@
use crate::language::*; //#
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Drive, DriveMut)] //#
pub enum Type {
    Unit,
    Bool,
    Str,
    TraitSelf,
    Ref(Option<Lifetime>, Mutability, Box<Type>),
}
