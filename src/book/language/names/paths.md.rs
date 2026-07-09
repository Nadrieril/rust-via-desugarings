use crate::language::*; //#
//@ # Paths
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ A path is a sequence of one or more path segments separated by `::` tokens.
//@
//@ ## Types of paths
//@
//@ ### Simple paths
//@
//@ ```grammar
//@ SimplePath -> Path: path=IDENTIFIER
//@     => path
//@ ```
pub type Path = Identifier;
