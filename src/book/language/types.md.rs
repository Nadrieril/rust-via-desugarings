//@ # Types
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ Type:
//@     | `bool` => Type::Bool
//@     | `str` => Type::Str
//@     | `(` types=TupleTypes? `)` => Type::Tuple(types.unwrap_or_default())
//@     | `Self` => Type::TraitSelf
//@     | `&` lifetime=Lifetime? m=Mutability ty=Type => Type::Ref(lifetime, m, Box::new(ty))
//@
//@ TupleTypes -> Vec<Type>:
//@     types=(Type `,`)+ last=Type?
//@     => types.into_iter().chain(last).collect()
//@ ```
//@
use crate::language::*; //#
#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)] //#
pub enum Type {
    Bool,
    Str,
    Tuple(Vec<Type>),
    TraitSelf,
    Ref(Option<Lifetime>, Mutability, Box<Type>),
}

impl Type {
    pub fn mk_unit() -> Type {
        Type::Tuple(Vec::new())
    }
}
