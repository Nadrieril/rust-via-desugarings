//@ # The Language
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ In this section we define the syntactic components of the Rust language,
//@ along with their grammar.
//@ For now this describes only a very small subset of the full language.
//@
use crate::language::*; //#
//@
//@ A program consists in a list of items.
//@
//@ ```grammar
//@ Program:
//@     functions=Function*
//@     => Program { functions }
//@ ```
//@
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

//@ ## Misc
//@
//@ Some syntactic elements we haven't fleshed out yet.
//@
//@ ```grammar
//@ Identifier: IDENTIFIER
//@     => IDENTIFIER
//@ ```
//@
pub type Identifier = String;

//@ ```grammar
//@ Abi: STRING_LITERAL
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

//@ ```grammar
//@ GenericParams: UNSUPPORTED
//@     => GenericParams {}
//@ ```
//@
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GenericParams {}

//@ ```grammar
//@ WhereClauses: UNSUPPORTED
//@     => WhereClauses {}
//@ ```
//@
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WhereClauses {}

//@ ```grammar
//@ OuterAttribute: UNSUPPORTED
//@     => OuterAttribute {}
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OuterAttribute {}

//@ ```grammar
//@ Lifetime: LIFETIME
//@     => Lifetime {}
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifetime {}

//@ ```grammar
//@ PatternNoTopAlt:
//@     | IDENTIFIER => PatternNoTopAlt {}
//@     | `_` => PatternNoTopAlt {}
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternNoTopAlt {}
