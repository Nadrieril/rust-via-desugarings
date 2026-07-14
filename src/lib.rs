#![feature(deref_patterns)]
#![allow(incomplete_features)]
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
#[cfg(not(target_arch = "wasm32"))]
pub use desugarings::formality::{check_with_formality, translate_to_formality};
#[cfg(not(target_arch = "wasm32"))]
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
    Formality(String),
    Internal(String),
    MiniRust(String),
}

impl std::error::Error for CompilationError {}
impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::Parse(msg)
            | CompilationError::Desugaring(msg)
            | CompilationError::Formality(msg)
            | CompilationError::MiniRust(msg) => write!(f, "{msg}"),
            CompilationError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

pub fn parse_desugar_and_print_program(input: &str) -> Result<String, CompilationError> {
    let program = parser::parse_program(input)?;
    let program = desugar(program)?;
    Ok(print_program(&program))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_desugar_and_run_program(input: &str) -> Result<String, CompilationError> {
    let program = parser::parse_program(input)?;
    let program = desugar(program)?;
    check_and_run(&program)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn check_and_run(program: &Program) -> Result<String, CompilationError> {
    check_with_formality(program)?;
    run_in_minirust(program)
}

pub mod interactive_examples {
    //! Helpers to make interactive desugaring examples: a call to `interactive_example!` does two
    //! things: on the Rust side, it registers a mapping from that file location to a particular
    //! desugaring function, and makes it available to call from wasm. On the mdbook side, that
    //! macro call shape is recognized and turned into an interactive element where you can edit
    //! code and see the desugared result next to it live.
    use itertools::Itertools;

    use super::*;

    pub struct InteractiveExample {
        pub id: &'static str,
        pub step: fn(&mut Program) -> Result<(), CompilationError>,
    }

    inventory::collect!(InteractiveExample);

    #[macro_export]
    macro_rules! interactive_example {
        ($step:path, $($code:tt)*) => {
            inventory::submit! {
                $crate::interactive_examples::InteractiveExample {
                    id: concat!(file!(), ":", line!()),
                    step: $step,
                }
            }
        };
    }

    fn interactive_example_by_id(
        example_id: &str,
    ) -> Result<&'static InteractiveExample, CompilationError> {
        for example in inventory::iter::<InteractiveExample> {
            if example.id == example_id {
                return Ok(example);
            }
        }
        let supported = inventory::iter::<InteractiveExample>
            .into_iter()
            .map(|example| example.id)
            .sorted()
            .format(", ");
        Err(CompilationError::Internal(format!(
            "unknown interactive example `{example_id}`; expected one of: {}",
            supported,
        )))
    }

    fn parse_apply_interactive_example_and_print_program(
        example_id: &str,
        input: &str,
    ) -> Result<String, CompilationError> {
        let example = interactive_example_by_id(example_id)?;
        let mut program = parser::parse_program(input)?;
        (example.step)(&mut program)?;
        Ok(print_program(&program))
    }

    // Hack to make sure `inventory` works reliably on wasm.
    #[cfg(target_family = "wasm")]
    fn ensure_wasm_ctors() {
        static CTORS: std::sync::Once = std::sync::Once::new();
        unsafe extern "C" {
            fn __wasm_call_ctors();
        }
        CTORS.call_once(|| unsafe {
            __wasm_call_ctors();
        });
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn interactive_desugar_example(
        example_id: &str,
        input: &str,
    ) -> Result<String, wasm_bindgen::JsValue> {
        ensure_wasm_ctors();
        parse_apply_interactive_example_and_print_program(example_id, input)
            .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
    }
}
