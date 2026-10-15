fn main() {
    let x: bool = true;
    {
        let x: bool = x;
        print(x);
    }
}
