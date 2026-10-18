//@ expect-rejection: Invalid @ Desugaring
// A local isn't visible after the end of its block.
fn main() {
    {
        let x: bool = true;
    }
    print(x);
}
