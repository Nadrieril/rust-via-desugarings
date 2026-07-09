//@ # The Language
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ In this section we define the syntactic components of the Rust language,
//@ along with their grammar.
//@ For now this describes only a very small subset of the full language.
//@
//@ A program consists in a list of items.
//@
//@ ```grammar
//@ Program:
//@     items=Item*
//@     => Program { items }
//@ ```
//@
#[derive(Debug, Default, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct Program {
    pub items: Vec<Item>,
}

//@ ## Misc
//@
//@ Some syntactic elements we haven't fleshed out yet.
//@
//@ ```grammar
//@ Identifier -> String: IDENTIFIER
//@     => IDENTIFIER
//@ ```
//@
pub type Identifier = String;

//@ ```grammar
//@ Abi -> String: STRING_LITERAL
//@     => STRING_LITERAL
//@ ```
//@
pub type Abi = String;

//@ ```grammar
//@ Mutability:
//@     is_mut=`mut`?
//@     => if is_mut.is_some() { Mutability::Mutable } else { Mutability::Immutable }
//@ ```
//@
#[derive(Debug, Clone, Copy, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum Mutability {
    Mutable,
    Immutable,
}

//@ ```grammar
//@ GenericParams: UNSUPPORTED
//@     => GenericParams {}
//@ ```
//@
#[derive(Debug, Default, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct GenericParams {}

//@ ```grammar
//@ WhereClauses: UNSUPPORTED
//@     => WhereClauses {}
//@ ```
//@
#[derive(Debug, Default, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct WhereClauses {}

//@ ```grammar
//@ OuterAttribute: UNSUPPORTED
//@     => OuterAttribute {}
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct OuterAttribute {}

//@ ```grammar
//@ InnerAttribute: UNSUPPORTED
//@     => InnerAttribute {}
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct InnerAttribute {}

//@ ```grammar
//@ Lifetime: LIFETIME
//@     => Lifetime {}
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct Lifetime {}

//@ ```grammar
//@ Visibility:
//@     | `pub` => Visibility::Pub,
//@     | `pub` `(` `crate` `)` => Visibility::PubCrate,
//@     | `pub` `(` `self` `)` => Visibility::PubSelf,
//@     | `pub` `(` `super` `)` => Visibility::PubSuper,
//@     | `pub` `(` `in` path=SimplePath `)` => Visibility::InPath(path),
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum Visibility {
    Pub,
    PubCrate,
    PubSelf,
    PubSuper,
    InPath(Path),
}

//@ ```grammar
//@ PatternNoTopAlt -> Pattern:
//@     | name=IDENTIFIER => Pattern::Identifier(name)
//@     | `_` => Pattern::Wildcard
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum Pattern {
    Identifier(Identifier),
    Wildcard,
}

//@ ## Submodules
pub use derive_generic_visitor::{Drive, DriveMut}; //#
#[path = "expressions.md.rs"]
pub mod expressions;
#[path = "items.md.rs"]
pub mod items;
#[path = "lexing.md.rs"]
pub mod lexing;
#[path = "names.md.rs"]
pub mod names;
#[path = "print.md.rs"]
pub mod print;
#[path = "statements.md.rs"]
pub mod statements;
#[path = "types.md.rs"]
pub mod types;
#[path = "visitor.md.rs"]
pub mod visitor;

pub use expressions::*;
pub use items::*;
pub use lexing::*;
pub use names::*;
pub use print::*;
pub use statements::*;
pub use types::*;
pub use visitor::*;
