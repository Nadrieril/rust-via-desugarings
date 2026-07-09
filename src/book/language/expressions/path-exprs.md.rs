use crate::language::*; //#
//@ # Path Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ PathExpression -> Identifier: variable=Identifier => variable
//@ ```
pub type PathExpression = Identifier;
