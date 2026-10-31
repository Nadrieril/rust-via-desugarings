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
        Rejection,
        language::{Program, Token},
    };
    use logos::Logos;

    include!(concat!(env!("OUT_DIR"), "/parser.rs"));

    impl From<ParseError> for Rejection {
        fn from(value: ParseError) -> Self {
            Rejection::unclassified(format!("{value:?}"))
        }
    }

    pub fn parse_program(input: &str) -> Result<Program, Rejection> {
        let mut lexer = Token::lexer(input);
        let mut context = ProgramContext::with_default_userdata();
        for token in lexer.by_ref() {
            match token {
                Ok(token) => context.feed(token)?,
                Err(()) => {
                    let offset = lexer.span().start;
                    let ch = lexer.slice().chars().next().unwrap_or('\0');
                    return Err(Rejection::unclassified(format!(
                        "unexpected character {ch:?} at byte {offset}"
                    )));
                }
            }
        }
        let parses: Vec<_> = context.accept_all()?.collect();
        match parses.as_slice() {
            [(program, _data)] => Ok(program.clone()),
            [] => Err(Rejection::unclassified("no valid parse".to_owned())),
            parses => Err(Rejection::unclassified(format!(
                "ambiguous parse: found {} valid parses, such as:\nparse 1:\n{}\n\nparse 2:\n{}",
                parses.len(),
                parses[0].0,
                parses[1].0,
            ))),
        }
    }
}

/// Why a program was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionKind {
    /// The program isn't valid Rust.
    Invalid,
    /// The program uses a part of the language which isn't yet handled properly by the executable
    /// part of this book.
    Unimplemented,
    /// There's a mistake in the executable implementation of this book.
    InternalError,
    /// The program is valid Rust, but failed at runtime (for example, it has undefined behaviour).
    RuntimeError,
    /// The stage which rejected the program can't tell whether the program isn't valid Rust or
    /// uses a part of the language which isn't yet modelled.
    Unclassified,
}

/// The stage of the pipeline which rejected a program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Parsing using the grammar used in the executable part of this book.
    Parse,
    /// The desugaring steps.
    Desugaring,
    /// Translating the desugared program into a-mir-formality's language.
    FormalityTranslation,
    /// a-mir-formality's checks (such as borrow-checking).
    FormalityCheck,
    /// Translating the desugared program into MiniRust.
    MiniRustTranslation,
    /// Running the program in MiniRust.
    MiniRustExecution,
}

/// A rejection reported from within a single stage of the pipeline.
///
/// The code driving the pipeline turns this into a [`ProgramRejection`] by recording which stage it
/// came from.
#[derive(Debug)]
pub struct Rejection {
    pub kind: RejectionKind,
    pub message: String,
}

impl Rejection {
    pub fn new(kind: RejectionKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Says that the program isn't valid Rust.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(RejectionKind::Invalid, message)
    }

    /// Says that the program uses something which isn't yet modelled.
    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self::new(RejectionKind::Unimplemented, message)
    }

    /// Says that there's a mistake in the executable implementation of this book.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(RejectionKind::InternalError, message)
    }

    /// Says that the program is valid Rust, but failed at runtime.
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(RejectionKind::RuntimeError, message)
    }

    /// Says that the program was rejected, without saying whether it isn't valid Rust or uses
    /// something which isn't yet modelled.
    pub fn unclassified(message: impl Into<String>) -> Self {
        Self::new(RejectionKind::Unclassified, message)
    }

    pub fn at(self, stage: Stage) -> ProgramRejection {
        ProgramRejection {
            stage,
            kind: self.kind,
            message: self.message,
        }
    }
}

/// Records that a program was rejected: which stage rejected it, and why.
#[derive(Debug)]
pub struct ProgramRejection {
    pub stage: Stage,
    pub kind: RejectionKind,
    pub message: String,
}

impl std::error::Error for ProgramRejection {}
impl std::fmt::Display for ProgramRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = &self.message;
        match self.kind {
            RejectionKind::Invalid | RejectionKind::RuntimeError | RejectionKind::Unclassified => {
                write!(f, "{msg}")
            }
            RejectionKind::Unimplemented => write!(f, "not yet implemented: {msg}"),
            RejectionKind::InternalError => write!(f, "internal error: {msg}"),
        }
    }
}

trait AtStage<T> {
    fn at_stage(self, stage: Stage) -> Result<T, ProgramRejection>;
}

impl<T> AtStage<T> for Result<T, Rejection> {
    fn at_stage(self, stage: Stage) -> Result<T, ProgramRejection> {
        self.map_err(|rejection| rejection.at(stage))
    }
}

/// Parses a program.
pub fn parse(input: &str) -> Result<Program, ProgramRejection> {
    parser::parse_program(input).at_stage(Stage::Parse)
}

/// Applies all the desugaring steps to a program.
pub fn desugar_program(program: Program) -> Result<Program, ProgramRejection> {
    desugar(program).at_stage(Stage::Desugaring)
}

pub fn parse_desugar_and_print_program(input: &str) -> Result<String, ProgramRejection> {
    let program = desugar_program(parse(input)?)?;
    Ok(print_program(&program))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_desugar_and_run_program(input: &str) -> Result<String, ProgramRejection> {
    let program = desugar_program(parse(input)?)?;
    check_and_run(&program)
}

/// Checks a desugared program using a-mir-formality, then runs it using MiniRust.
#[cfg(not(target_arch = "wasm32"))]
pub fn check_and_run(program: &Program) -> Result<String, ProgramRejection> {
    let crates = translate_to_formality(program).at_stage(Stage::FormalityTranslation)?;
    check_with_formality(crates).at_stage(Stage::FormalityCheck)?;
    let program = translate_to_minirust(program).at_stage(Stage::MiniRustTranslation)?;
    run_in_minirust(program).at_stage(Stage::MiniRustExecution)
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
        pub step: fn(&mut Program) -> Result<(), Rejection>,
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

    fn interactive_example_by_id(example_id: &str) -> Result<&'static InteractiveExample, String> {
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
        Err(format!(
            "internal error: unknown interactive example `{example_id}`; expected one of: {}",
            supported,
        ))
    }

    #[allow(unused)] // Used in wasm
    fn parse_apply_interactive_example_and_print_program(
        example_id: &str,
        input: &str,
    ) -> Result<String, String> {
        let example = interactive_example_by_id(example_id)?;
        let mut program = parse(input).map_err(|rejection| rejection.to_string())?;
        (example.step)(&mut program)
            .at_stage(Stage::Desugaring)
            .map_err(|rejection| rejection.to_string())?;
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
            .map_err(|error| wasm_bindgen::JsValue::from_str(&error))
    }
}
