fn main() {
    {
        fn f1() {}
    }
    fn f2() {
        fn f3() {}
    }
    fn f4() {}
}

fn foo() {
    fn f5() {}
}
