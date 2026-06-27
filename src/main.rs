use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    match parse_and_print_program(&input) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn parse_and_print_program(
    input: &str,
) -> Result<String, rust_via_desugarings::parser::ParseError<'_>> {
    rust_via_desugarings::parser::parse_program(input)
        .map(|program| rust_via_desugarings::print_program(&program))
}
