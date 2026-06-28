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
//@ ```rustylr
//@ Program(Program)
//@     : functions=Function* {
//@         Program { functions }
//@     }
//@     ;
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
//@ ```rustylr
//@ Identifier(Identifier)
//@     : identifier {
//@         let Token::Identifier(identifier) = identifier else {
//@             unreachable!("expected identifier token")
//@         };
//@         identifier
//@     }
//@     ;
//@ ```
//@
pub type Identifier = String;

//@ ```rustylr
//@ Mutability(Mutability)
//@     : is_mut=mut_? {
//@         if is_mut.is_some() { Mutability::Mutable } else { Mutability::Immutable }
//@     }
//@     ;
//@ ```
//@
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

//@ ```rustylr
//@ GenericParams(GenericParams)
//@     : unsupported! { GenericParams }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GenericParams;

//@ ```rustylr
//@ WhereClause(WhereClause)
//@     : unsupported! { WhereClause }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhereClause;

//@ ```rustylr
//@ OuterAttribute(OuterAttribute)
//@     : unsupported! { OuterAttribute }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OuterAttribute;

//@ ```rustylr
//@ Lifetime(Lifetime)
//@     : lifetime! {
//@         Lifetime
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lifetime;

//@ ```rustylr
//@ PatternNoTopAlt(PatternNoTopAlt)
//@     : identifier! {
//@         PatternNoTopAlt
//@     }
//@     | underscore! {
//@         PatternNoTopAlt
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternNoTopAlt;
