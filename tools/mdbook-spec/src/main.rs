fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("supports") => return,
        Some(arg) => {
            eprintln!("unknown argument: {arg}");
            std::process::exit(1);
        }
        None => {}
    }

    if let Err(err) = mdbook_spec::handle_preprocessing() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
