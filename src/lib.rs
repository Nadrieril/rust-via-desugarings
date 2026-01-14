// This file is here because
// 1. for editor integration the literate Rust files should be placed in the filesystem like normal
//    modules;
// 2. mdbook refuses to read files outside its directory;
// 3. mdbook indexes everything it sees, including cargo's `target/` directory if there's one.
//
// Therefore the rust project root must be a parent of the mdbook root.
#![feature(rustc_private)]
#![feature(if_let_guard)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_index;
extern crate rustc_middle;
extern crate rustc_span;

pub mod desugar;
pub mod print;

pub use desugar::Body;
