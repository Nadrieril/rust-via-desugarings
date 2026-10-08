fn main() {
    let x: bool = {
        fn f() -> bool {
            true
        }
        f()
    };
    print(x);
}
