// This file is here because
// 1. for editor integration the literate Rust files should be placed in the filesystem like normal
//    modules;
// 2. mdbook refuses to read files outside its directory;
// 3. mdbook indexes everything it sees, including cargo's `target/` directory if there's one.
//
// Therefore the rust project root must be a parent of the mdbook root.
#[path = "book/language/mod.rs"]
pub mod language;
pub use language::{Program, print_program};

#[path = "book/pipeline/mod.rs"]
pub mod desugarings;
pub use desugarings::overview::desugar;

pub mod parser {
    // Note: lalrpop is unfortunately not that close to the kind of grammar the Reference has.
    // Idea: make custom grammar syntax that maps onto rustemo (GLR parser) + custom disambiguators
    // to get as close as possible to the Reference kind of shape. With a lalrpop flavor of
    // giving names + the rust action probably.
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));

    pub type ParseError<'input> =
        lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>;

    pub fn parse_program(input: &str) -> Result<Program, ParseError<'_>> {
        ProgramParser::new().parse(input)
    }
}

pub fn parse_desugar_and_print_program(input: &str) -> Result<String, parser::ParseError<'_>> {
    let mut program = parser::parse_program(input)?;
    desugar(&mut program);
    Ok(print_program(&program))
}
