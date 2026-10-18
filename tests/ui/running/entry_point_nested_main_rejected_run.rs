//@ run
//@ expect-rejection: Invalid @ MiniRustTranslation
// A nested function named `main` isn't the program's entry point.
fn foo() {
    fn main() {
        print(true);
    }
}
