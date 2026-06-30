// This file is here because
// 1. for editor integration the literate Rust files should be placed in the filesystem like normal
//    modules;
// 2. mdbook refuses to read files outside its directory;
// 3. mdbook indexes everything it sees, including cargo's `target/` directory if there's one.
//
// Therefore the rust project root must be a parent of the mdbook root.
#[path = "book/language/overview.md.rs"]
pub mod language;
pub use language::{Program, print_program};

#[path = "book/pipeline/overview.md.rs"]
pub mod desugarings;
pub use desugarings::desugar;
pub use desugarings::minirust::{run_in_minirust, translate_to_minirust};

pub mod parser {
    use crate::{
        CompilationError,
        language::{Program, Token},
    };
    use logos::Logos;

    include!(concat!(env!("OUT_DIR"), "/parser.rs"));

    impl From<ParseError> for CompilationError {
        fn from(value: ParseError) -> Self {
            CompilationError::Parse(format!("{value:?}"))
        }
    }

    pub fn parse_program(input: &str) -> Result<Program, CompilationError> {
        let mut lexer = Token::lexer(input);
        let mut context = ProgramContext::with_default_userdata();
        for token in lexer.by_ref() {
            match token {
                Ok(token) => context.feed(token)?,
                Err(()) => {
                    let offset = lexer.span().start;
                    let ch = lexer.slice().chars().next().unwrap_or('\0');
                    return Err(CompilationError::Parse(format!(
                        "unexpected character {ch:?} at byte {offset}"
                    )));
                }
            }
        }
        let parses: Vec<_> = context.accept_all()?.collect();
        match parses.as_slice() {
            [(program, _data)] => Ok(program.clone()),
            [] => Err(CompilationError::Parse("no valid parse".to_owned())),
            parses => Err(CompilationError::Parse(format!(
                "ambiguous parse: found {} valid parses, such as:\nparse 1:\n{}\n\nparse 2:\n{}",
                parses.len(),
                parses[0].0,
                parses[1].0,
            ))),
        }
    }
}

#[derive(Debug)]
pub enum CompilationError {
    Parse(String),
    Desugaring(String),
    MiniRust(String),
}

impl std::error::Error for CompilationError {}
impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (CompilationError::Parse(msg)
        | CompilationError::Desugaring(msg)
        | CompilationError::MiniRust(msg)) = self;
        write!(f, "{msg}")
    }
}

pub fn parse_desugar_and_print_program(input: &str) -> Result<String, CompilationError> {
    let program = parser::parse_program(input)?;
    let program = desugar(program)?;
    Ok(print_program(&program))
}

pub fn parse_desugar_and_run_program(input: &str) -> Result<String, CompilationError> {
    let program = parser::parse_program(input)?;
    let program = desugar(program)?;
    run_in_minirust(&program)
}
