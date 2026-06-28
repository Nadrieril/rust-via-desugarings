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
    use crate::language::{Program, Token};
    use logos::Logos;
    use std::{
        error::Error,
        fmt::{self, Debug},
    };

    include!(concat!(env!("OUT_DIR"), "/parser.rs"));

    #[derive(Debug)]
    pub struct ParseProgramError(String);

    impl Error for ParseProgramError {}
    impl fmt::Display for ParseProgramError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl From<ParseError> for ParseProgramError {
        fn from(value: ParseError) -> Self {
            ParseProgramError(format!("{value:?}"))
        }
    }

    pub fn parse_program(input: &str) -> Result<Program, ParseProgramError> {
        let mut lexer = Token::lexer(input);
        let mut context = ProgramContext::with_default_userdata();
        for token in lexer.by_ref() {
            match token {
                Ok(token) => context.feed(token)?,
                Err(()) => {
                    let offset = lexer.span().start;
                    let ch = lexer.slice().chars().next().unwrap_or('\0');
                    return Err(ParseProgramError(format!(
                        "unexpected character {ch:?} at byte {offset}"
                    )));
                }
            }
        }
        let parses: Vec<_> = context.accept_all()?.collect();
        match parses.as_slice() {
            [(program, _data)] => Ok(program.clone()),
            [] => Err(ParseProgramError("no valid parse".to_owned())),
            parses => Err(ParseProgramError(format!(
                "ambiguous parse: found {} valid parses, such as:\nparse 1:\n{}\n\nparse 2:\n{}",
                parses.len(),
                parses[0].0,
                parses[1].0,
            ))),
        }
    }
}

pub fn parse_desugar_and_print_program(input: &str) -> Result<String, parser::ParseProgramError> {
    let mut program = parser::parse_program(input)?;
    desugar(&mut program);
    Ok(print_program(&program))
}
