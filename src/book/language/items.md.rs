//@ # Items
//@
//@ > This section is a work-in-progress experiment about making the book executable.
use crate::language::*; //#

#[path = "items/functions.md.rs"]
pub mod functions;
pub use functions::*;
//@
//@ ```grammar
//@ Item:
//@     attrs=OuterAttribute* visibility=Visibility? kind=ItemKind
//@     => Item { attrs, visibility, kind }
//@
//@ ItemKind:
//@     function=Function => ItemKind::Function(function)
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub struct Item {
    pub attrs: Vec<OuterAttribute>,
    pub visibility: Option<Visibility>,
    pub kind: ItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum ItemKind {
    Function(Function),
}
