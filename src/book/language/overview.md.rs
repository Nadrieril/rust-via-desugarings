//@ # The Language
//@
//@ In this section we define the syntactic components of the Rust language,
//@ along with their grammar.
//@ For now this describes only a very small subset of the full language.
//@
use crate::language::*; //#
//@
//@ A program consists in a list of items.
//@
//@ ```lalrpop
//@ pub Program: Program = {
//@     <functions:Function*> => Program { functions },
//@ };
//@ ```
//@
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

//@ ## Misc
//@
//@ Some syntactic elements we haven't fleshed out yet.
//@
//@ ```lalrpop
//@ Identifier: Identifier = {
//@     <name:r"[A-Za-z_][A-Za-z0-9_]*"> => name.to_owned(),
//@ };
//@ ```
//@
pub type Identifier = String;

//@ ```lalrpop
//@ GenericParams: GenericParams = {
//@     "__generic_params_unsupported__" => GenericParams,
//@ };
//@ ```
//@
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GenericParams;

//@ ```lalrpop
//@ WhereClause: WhereClause = {
//@     "__where_clause_unsupported__" => WhereClause,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhereClause;

//@ ```lalrpop
//@ OuterAttribute: OuterAttribute = {
//@     "__outer_attribute_unsupported__" => OuterAttribute,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OuterAttribute;

//@ ```lalrpop
//@ Lifetime: Lifetime = {
//@     "__lifetime_unsupported__" => Lifetime,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lifetime;

//@ ```lalrpop
//@ PatternNoTopAlt: PatternNoTopAlt = {
//@     "__pattern_no_top_alt_unsupported__" => PatternNoTopAlt,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternNoTopAlt;

//@ ```lalrpop
//@ STRING_LITERAL: String = {
//@     <literal:r#""[^"]*""#> => literal.to_owned(),
//@ };
//@
//@ RAW_STRING_LITERAL: String = {
//@     "__raw_string_literal_unsupported__" => "__raw_string_literal_unsupported__".to_owned(),
//@ };
//@ ```
