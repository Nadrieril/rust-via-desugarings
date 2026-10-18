//@ expect-rejection: Invalid @ Desugaring
// A parameter isn't visible outside its function.
fn f1(x: bool) {}

fn main() {
    print(x);
}
